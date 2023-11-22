// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! 'Delegates' are the core abstraction of the Spelldawn rules engine.
//!
//! There are two types of delegates: 'Events' and 'Queries'. Event delegates
//! allow cards to respond to specific events which occur during a game, such as
//! taking an action when a card is played or at the start of a turn.
//!
//! Query delegates allow cards to read & intercept requests for game data --
//! for example, the 'can play card' query is used to determine whether a card
//! can be legally played, a card delegate might add custom logic to determine
//! when it can be played. Similarly, the 'attack value' query is used to
//! determine the attack strength of a weapon; a delegate could intercept this
//! request to change the attack power of a given card.
//!
//! Every delegate in the game is run for every applicable event. Even when
//! cards are shuffled into a player's deck, their delegates are invoked. Each
//! delegate has a [RequirementFn] which needs to return true when the delegate
//! should run.
//!
//! Delegate callbacks are always raw function pointers, which means they
//! *cannot be closures* to state from their enclosing scope. When I last
//! profiled it, switching delegates to use Arc<dyn Fn()> closures resulted
//! in game simulation code running 25% slower.
//!
//! Delegate enum members automatically have an associated struct generated for
//! them by the [DelegateEnum] macro, which is the name of the enum variant with
//! the prefix `Event` or `Query`, e.g. [DawnEvent] for `Delegate::Dawn`.
//!
//! # Example Generated Code
//! We generate approximately the following code for each delegate enum value:
//!
//! ```
//! #[derive(Debug, Copy, Clone)]
//! pub struct OnDawnEvent(pub TurnNumber);
//!
//! impl EventData<TurnNumber> for OnDawnEvent {
//!     fn data(&self) -> TurnNumber {
//!         self.0
//!     }
//!
//!     fn extract(delegate: &Delegate) -> Option<EventDelegate<TurnNumber>> {
//!         match delegate {
//!             Delegate::OnDawn(d) => Some(*d),
//!             _ => None,
//!         }
//!     }
//! }
//! ```

#![allow(clippy::use_self)] // Required to use EnumKind

use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

use anyhow::Result;
use enum_kinds::EnumKind;
use macros::DelegateEnum;
use serde::{Deserialize, Serialize};

#[allow(unused)] // Used in rustdocs
use crate::card_definition::Cost;
use crate::card_definition::Resonance;
#[allow(unused)] // Used in rustdocs
use crate::card_definition::{AttackBoost, CardStats};
use crate::card_name::CardMetadata;
#[allow(unused)] // Used in rustdocs
use crate::card_state::{CardData, CardPosition};
use crate::continuous_visual_effect::ContinuousDisplayEffect;
use crate::game_actions::{CardTarget, GameStateAction};
use crate::game_state::GameState;
use crate::primitives::{
    AbilityId, ActionCount, AttackValue, BreachValue, CardId, CurseCount, HasAbilityId, HasCardId,
    HasRaidId, HasRoomId, HasSide, HealthValue, InitiatedBy, ManaValue, MinionEncounterId, RaidId,
    RoomAccessId, RoomId, ShieldValue, Side, TurnNumber,
};
use crate::raid_data::PopulateAccessPromptSource;
use crate::text::TextElement;

/// Identifies the context for a given request to a delegate: which player,
/// card, & card ability owns the delegate.
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Scope {
    /// Ability which owns this delegate.
    ability_id: AbilityId,
    /// Metadata for the this card
    metadata: CardMetadata,
}

impl Scope {
    pub fn new(ability_id: AbilityId, metadata: CardMetadata) -> Self {
        Self { ability_id, metadata }
    }

    /// Player who owns this scope
    pub fn side(&self) -> Side {
        self.card_id().side
    }

    /// Ability which owns this scope
    pub fn ability_id(&self) -> AbilityId {
        self.ability_id
    }

    /// Card which owns this scope
    pub fn card_id(&self) -> CardId {
        self.ability_id.card_id
    }

    pub fn metadata(&self) -> CardMetadata {
        self.metadata
    }

    pub fn is_upgraded(&self) -> bool {
        self.metadata.is_upgraded
    }

    /// Returns one of two values based on whether the card is upgraded
    pub fn upgrade<T>(&self, normal: T, upgraded: T) -> T {
        self.metadata.upgrade(normal, upgraded)
    }

    /// Builds an [InitiatedBy] struct for this scope's ability.
    pub fn initiated_by(&self) -> InitiatedBy {
        InitiatedBy::Ability(self.ability_id)
    }
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.ability_id)
    }
}

impl HasAbilityId for Scope {
    fn ability_id(&self) -> AbilityId {
        self.ability_id
    }
}

impl HasSide for Scope {
    fn side(&self) -> Side {
        self.ability_id.card_id.side
    }
}

/// Predicate to determine whether a delegate should run, taking contextual
/// information `T`.
pub type RequirementFn<T> = fn(&GameState, Scope, &T) -> bool;
/// Function to mutate game state in response to an event, taking contextual
/// information `T`.
pub type MutationFn<T> = fn(&mut GameState, Scope, &T) -> Result<()>;
/// Function to intercept a query for game information, taking contextual
/// information `T` and the current query value `R`.
pub type TransformationFn<T, R> = fn(&GameState, Scope, &T, R) -> R;

/// Delegate which responds to a given game event and mutates game state in
/// response.
#[derive(Copy, Clone)]
pub struct EventDelegate<T> {
    /// Should return true if this delegate's `mutation`.
    pub requirement: RequirementFn<T>,
    /// Modifies the current [GameState] in response to the associated event.
    pub mutation: MutationFn<T>,
}

impl<T> EventDelegate<T> {
    pub fn new(requirement: RequirementFn<T>, mutation: MutationFn<T>) -> Self {
        Self { requirement, mutation }
    }
}

/// Delegate which intercepts and transforms a query for game information.
#[derive(Copy, Clone)]
pub struct QueryDelegate<T, R> {
    /// Should return true if this delegate's `transformation` should run.
    pub requirement: RequirementFn<T>,
    /// Function which takes contextual data and the current value of some piece
    /// of game information and returns a transformed value for this
    /// information.
    pub transformation: TransformationFn<T, R>,
}

impl<T, R> QueryDelegate<T, R> {
    pub fn new(requirement: RequirementFn<T>, transformation: TransformationFn<T, R>) -> Self {
        Self { requirement, transformation }
    }
}

/// A Flag is a variant of boolean which typically indicates whether some game
/// action can currently be taken. Flags have a 'default' state, which is the
/// value of the flag based on standard game rules, and an 'override' state,
/// which is a value set by specific delegates. An override of 'false' takes
/// precedence over an override of 'true'.
///
/// For example, the 'CanPlay' delegate will be invoked with
/// `Flag::Default(false)` if a card cannot currently be played according to the
/// standard game rules (sufficient mana available, correct player's turn, etc).
/// A delegate could transform this via `allow()` to allow the card
/// to be played. A second delegate could set `disallow()` to prevent
/// the card from being played, and this would take priority.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Flag {
    /// Initial value of this flag
    Default(bool),
    /// Override for this flag set by a delegate
    Override(bool),
}

impl Flag {
    pub fn new(value: bool) -> Self {
        Self::Default(value)
    }

    /// Allows some player action or event that would not otherwise happen. This
    /// has priority over base game rules, but is superseded in turn by
    /// [Self::disallow] and [Self::add_constraint].
    pub fn allow(self) -> Self {
        self.override_unconditionally(true)
    }

    /// Prevents some player action or event from happening. This is the highest
    /// priority option and cannot be superseded.
    pub fn disallow(self) -> Self {
        self.override_unconditionally(false)
    }

    /// Overrides this flag if `value` is false. This is used to modify
    /// something that a player *could otherwise do* with an additional
    /// constraint that prevents it from happening. It cannot *expand* the
    /// scope where an event can happen.
    pub fn add_constraint(self, value: bool) -> Self {
        if value {
            self
        } else {
            self.override_unconditionally(value)
        }
    }

    /// Overrides this flag if `value` is true. This is used to modify
    /// something that a player *could not* otherwise do with an additional
    /// capability. It expands the scope of where an action can happen, but
    /// cannot *restrict* anything that was already allowed.
    ///
    /// This has lower priority than [Self::add_constraint]. This behavior is
    /// sometimes described as the "can't beats can" rule.
    pub fn add_permission(self, value: bool) -> Self {
        if value {
            self.override_unconditionally(value)
        } else {
            self
        }
    }

    fn override_unconditionally(self, value: bool) -> Self {
        match self {
            Self::Default(_) => Self::Override(value),
            Self::Override(current) => Self::Override(current && value),
        }
    }
}

impl From<Flag> for bool {
    fn from(flag: Flag) -> Self {
        match flag {
            Flag::Default(value) | Flag::Override(value) => value,
        }
    }
}

/// Event data for when a card is played
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CardPlayed {
    pub card_id: CardId,
    pub target: CardTarget,
}

impl HasCardId for CardPlayed {
    fn card_id(&self) -> CardId {
        self.card_id
    }
}

/// Event data for when an ability is activated
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct AbilityActivated {
    pub ability_id: AbilityId,
    pub target: CardTarget,
}

impl AbilityActivated {
    pub fn card_id(&self) -> CardId {
        self.ability_id.card_id
    }
}

impl HasAbilityId for AbilityActivated {
    fn ability_id(&self) -> AbilityId {
        self.ability_id
    }
}

/// Event data for encounters between cards
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CardEncounter {
    /// Card initiating the interaction
    pub source: CardId,
    /// Card being targeted
    pub target: CardId,
}

impl CardEncounter {
    pub fn new(source: CardId, target: CardId) -> Self {
        Self { source, target }
    }
}

/// Event data when a raid is in progress
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct RaidEvent<T> {
    pub raid_id: RaidId,
    pub target: RoomId,
    pub minion_encounter_id: Option<MinionEncounterId>,
    pub room_access_id: Option<RoomAccessId>,
    pub data: T,
}

impl<T> HasRoomId for RaidEvent<T> {
    fn room_id(&self) -> RoomId {
        self.target
    }
}

impl<T> HasRaidId for RaidEvent<T> {
    fn raid_id(&self) -> RaidId {
        self.raid_id
    }
}

/// Event data when a weapon is used
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct UsedWeapon {
    pub weapon_id: CardId,
    pub target_id: CardId,
    /// Mana spent to use this weapon
    pub mana_spent: ManaValue,
    /// Attack value added to this weapon to defeat this minion
    pub attack_boost: AttackValue,
}

/// Event data when a card is scored
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct ScoreCard {
    pub player: Side,
    pub card_id: CardId,
}

impl HasCardId for ScoreCard {
    fn card_id(&self) -> CardId {
        self.card_id
    }
}

/// Result of a raid
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum RaidOutcome {
    Success,
    Failure,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct DealtDamage {
    pub source: AbilityId,
    pub amount: u32,
}

impl HasAbilityId for DealtDamage {
    fn ability_id(&self) -> AbilityId {
        self.source
    }
}

/// Actions to show the Champion during combat in addition to their weapon
/// actions
#[derive(Clone, Debug)]
pub struct MinionCombatPrompt {
    /// Combat actions to show the Champion
    pub actions: Vec<GameStateAction>,
    /// Whether to show the default continue/don't use a weapon option
    pub include_no_action: bool,
}

/// Source from which a card has been discarded
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum DiscardedFrom {
    Deck,
    Hand,
}

/// Event information when a card is discarded from a deck or from hand
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct DiscardedCard {
    pub card_id: CardId,
    pub discarded_from: DiscardedFrom,
}

impl HasCardId for DiscardedCard {
    fn card_id(&self) -> CardId {
        self.card_id
    }
}

/// Event information when determining shield values for a minion
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct ShieldCardInfo {
    /// Minion to determine shield value for
    pub minion_id: CardId,
    /// Optionally, a weapon which is being used to attack this minion which may
    /// modify the shield value.
    pub weapon_id: Option<CardId>,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CanActivateAbility {
    pub ability_id: AbilityId,
    pub target: CardTarget,
}

impl HasAbilityId for CanActivateAbility {
    fn ability_id(&self) -> AbilityId {
        self.ability_id
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CardInfoElementKind {
    Informative,
    PositiveEffect,
    NegativeEffect,
}

/// Marker for a card's ongoing status
#[derive(Debug, Clone)]
pub struct CardStatusMarker {
    /// Ability which created the status. Used to determine the title, image and
    /// card frame used on the marker card.
    pub source: AbilityId,
    /// Whether this effect is positive, negative, etc. Used to determine color
    /// used in supplemental info.
    pub marker_kind: CardInfoElementKind,
    /// Text describing the status.
    pub text: Vec<TextElement>,
}

#[derive(Debug, Clone)]
pub struct ManaLostToOpponentAbility {
    pub side: Side,
    pub amount: ManaValue,
}

/// The core of the delegate pattern, used to identify which event or which
/// query this delegate wishes to respond to. Each enum variant here
/// automatically gets an associated struct value generated for it by the
/// [DelegateEnum] macro -- see module-level documentation for an example of
/// what this code looks like.
#[derive(EnumKind, DelegateEnum, Clone)]
#[enum_kind(DelegateKind, derive(Hash))]
pub enum Delegate {
    /// The Champion's turn begins
    Dawn(EventDelegate<TurnNumber>),
    /// The Overlord's turn begins
    Dusk(EventDelegate<TurnNumber>),
    /// A card is moved from a Deck position to a Hand position
    DrawCard(EventDelegate<CardId>),
    /// A user takes the explicit 'draw card' game action
    DrawCardAction(EventDelegate<CardId>),
    /// A card has been played via the Play Card action and has had its costs
    /// paid
    PlayCard(EventDelegate<CardPlayed>),
    /// A card has been moved from any non-arena zone to an arena zone.
    EnterArena(EventDelegate<CardId>),
    /// A card has been moved from a deck or hand to a discard pile.
    DiscardCard(EventDelegate<DiscardedCard>),
    /// A card is moved to the discard pile from anywhere
    MoveToDiscardPile(EventDelegate<CardId>),
    /// A card ability with a cost is activated
    ActivateAbility(EventDelegate<AbilityActivated>),
    /// A card Project is turned face up by paying its cost
    SummonProject(EventDelegate<CardId>),
    /// A minion card is turned face up.
    SummonMinion(EventDelegate<CardId>),
    /// A card is scored by the Overlord
    OverlordScoreCard(EventDelegate<CardId>),
    /// A card is scored by the Champion
    ChampionScoreCard(EventDelegate<CardId>),
    /// Either player scores a card
    ScoreCard(EventDelegate<ScoreCard>),
    /// A card is razed (discarded by paying its raze cost) by the Champion
    RazeCard(EventDelegate<CardId>),
    /// A Raid is initiated
    RaidStart(EventDelegate<RaidEvent<()>>),
    /// A summoned minion is about to be encountered during a raid
    ApproachMinion(EventDelegate<RaidEvent<CardId>>),
    /// A summoned minion is encountered during a raid
    EncounterMinion(EventDelegate<CardId>),
    /// A weapon has been used to defeat a minion
    UsedWeapon(EventDelegate<RaidEvent<UsedWeapon>>),
    /// A minion is defeated during an encounter by dealing damage to it equal
    /// to its health
    MinionDefeated(EventDelegate<CardId>),
    /// A minion's 'combat' ability is triggered during an encounter, typically
    /// because the minion was not defeated by the Champion.
    MinionCombatAbility(EventDelegate<CardId>),
    /// A minion finishes being encountered during a raid. Invokes regardless of
    /// whether the encounter was successful.
    EncounterEnd(EventDelegate<RaidId>),
    /// Minion encounters have been completed for a raid and the Access phase is
    /// about to start. The set of accessed cards has not yet been selected.
    RaidAccessStart(EventDelegate<RaidEvent<()>>),
    /// The set of cards accessed during a raid have been selected and written
    /// to `GameState`, but not 'on access' effects have yet triggered. This is
    /// the expected place to modify the set of accessed cards if it was not
    /// possible earlier.
    RaidAccessSelected(EventDelegate<RaidEvent<()>>),
    /// The game is about to populate an access prompt for the current set of
    /// accessed cards.
    WillPopulateAccessPrompt(EventDelegate<RaidEvent<PopulateAccessPromptSource>>),
    /// The card with the provided `card_id` has been accessed and revealed
    /// during a raid (in any zone), but not yet scored/acted on.
    CardAccess(EventDelegate<CardId>),
    /// Access phase has ended for a raid and the raid is about to end. Unlike
    /// the `RaidEnd` and `RaidSuccess` events, this does not trigger for raids
    /// where the access step was prevented (e.g. "instead of accessing that
    /// room, gain mana" type cards).
    RaidAccessEnd(EventDelegate<RaidEvent<()>>),
    /// A Custom Access raid has completed its access phase
    CustomAccessEnd(EventDelegate<InitiatedBy>),
    /// A Raid is completed, either successfully or unsuccessfully.
    ///
    /// Note that this is invoked before `game.data.raid` is cleared.
    RaidEnd(EventDelegate<RaidEvent<RaidOutcome>>),
    /// A raid has ended in failure.
    RaidFailure(EventDelegate<RaidEvent<()>>),
    /// A raid has ended in success.
    RaidSuccess(EventDelegate<RaidEvent<()>>),
    /// Stored mana is taken from a card
    StoredManaTaken(EventDelegate<CardId>),
    /// Damage is about to be dealt to the Champion player in a given amount.
    WillDealDamage(EventDelegate<DealtDamage>),
    /// Damage has been dealt to the Champion player (in the form of discarded
    /// cards).
    DealtDamage(EventDelegate<DealtDamage>),
    /// Curses are about to be given to the Champion player
    WillReceiveCurses(EventDelegate<CurseCount>),
    /// The Champion player has been given one or more curses
    CursesReceived(EventDelegate<CurseCount>),
    /// A card has been sacrificed by its owner
    CardSacrificed(EventDelegate<CardId>),
    /// A card has been revealed by an ability. This is a specific game action
    /// (described using the word "reveal" on card text) and does *not* include
    /// a card being made visible by normal game rules, e.g. during a raid.
    CardRevealed(EventDelegate<CardId>),
    /// Mana has been paid or lost due to an opponent ability.
    ManaLostToOpponentAbility(EventDelegate<ManaLostToOpponentAbility>),

    /// Query whether the indicated player can currently take the basic game
    /// action to spend an action point to draw a card.
    CanTakeDrawCardAction(QueryDelegate<Side, Flag>),
    /// Query whether the indicated player can currently take the basic game
    /// action to spend an action point to gain one mana
    CanTakeGainManaAction(QueryDelegate<Side, Flag>),
    /// Query whether a given card can currently be played.
    CanPlayCard(QueryDelegate<CardId, Flag>),
    /// Query whether a given ability can currently be activated.
    CanActivateAbility(QueryDelegate<CanActivateAbility, Flag>),
    /// Can a raid currently be started on the indicated room?
    CanInitiateRaid(QueryDelegate<RoomId, Flag>),
    /// Can the indicated player currently progress the indicated room?
    CanProgressRoom(QueryDelegate<RoomId, Flag>),
    /// Can the indicated card be progressed when the progress action is taken
    /// for a room?
    ///
    /// Note that Scheme cards can be progressed by default.
    CanProgressCard(QueryDelegate<CardId, Flag>),
    /// Can the source card currently be summoned?
    CanSummon(QueryDelegate<CardId, Flag>),
    /// Can the source card (typically a weapon) take an encounter action
    /// against the target card (typically a minion) during a raid?
    CanEncounterTarget(QueryDelegate<CardEncounter, Flag>),
    /// Can the source card (typically a weapon) apply an encounter
    /// action to defeat the target (typically a minion) during a raid?
    CanDefeatTarget(QueryDelegate<CardEncounter, Flag>),
    /// Can the Champion choose to not use a weapon ability when encountering
    /// the indicated minion card?
    CanUseNoWeapon(QueryDelegate<CardId, Flag>),
    /// Can an ongoing raid access cards any cards? If false the raid will
    /// immediately move to the 'end' state once it is successful.
    CanRaidAccessCards(QueryDelegate<RaidEvent<()>, Flag>),
    /// Can the Champion choose to use the 'End Raid' button to end the access
    /// phase of a raid?
    CanEndRaidAccessPhase(QueryDelegate<RaidId, Flag>),
    /// Should an 'end the raid' ability with the given ID be prevented?
    CanAbilityEndRaid(QueryDelegate<RaidEvent<AbilityId>, Flag>),
    /// Should a 'destroy' ability with the given ID be prevented?
    CanAbilityDestroyCard(QueryDelegate<AbilityId, Flag>),

    /// Query the current mana cost of a card. Invoked with [Cost::mana].
    ManaCost(QueryDelegate<CardId, Option<ManaValue>>),
    /// Query the current mana cost of an ability. Invoked with [Cost::mana].
    AbilityManaCost(QueryDelegate<AbilityId, Option<ManaValue>>),
    /// Query the current action cost of a card. Invoked with [Cost::actions].
    ActionCost(QueryDelegate<CardId, ActionCount>),
    /// Query the current attack value of a card. Invoked with
    /// [CardStats::base_attack] or 0.
    BaseAttack(QueryDelegate<CardId, AttackValue>),
    /// Query the amount of attack added each time a card's weapon boost ability
    /// is activated. Invokes with [AttackBoost::bonus].
    AttackBoostBonus(QueryDelegate<CardId, AttackValue>),
    /// Query the current health value of a card. Invoked with
    /// [CardStats::health] or 0.
    HealthValue(QueryDelegate<CardId, HealthValue>),
    /// Query the current shield value of a card. Invoked with
    /// [CardStats::shield] or 0.
    ShieldValue(QueryDelegate<ShieldCardInfo, ShieldValue>),
    /// Queries the current breach value of a card. Invoked with
    /// [CardStats::breach] or 0.
    BreachValue(QueryDelegate<CardId, BreachValue>),
    /// Queries the [Resonance] for a weapon or minion. Invoked with
    /// `CardConfig::resonance`.
    Resonance(QueryDelegate<CardId, Resonance>),
    /// Queries the current raze cost of a card. Invoked with
    /// [CardStats::raze_cost] or 0.
    RazeCost(QueryDelegate<CardId, BreachValue>),
    /// Get the number of actions a player gets at the start of their turn.
    StartOfTurnActions(QueryDelegate<Side, ActionCount>),
    /// Gets the number of cards the Champion player can access from the Vault
    /// during this raid
    VaultAccessCount(QueryDelegate<RaidId, u32>),
    /// Gets the number of cards the Champion player can access from the Sanctum
    /// during this raid
    SanctumAccessCount(QueryDelegate<RaidId, u32>),
    /// Queries the maximum hand size of a player. Invoked with the default
    /// maximum hand size.
    MaximumHandSize(QueryDelegate<Side, u32>),
    /// Queries continuous display effect for a card. This has no effect other
    /// than to display VFX on the cardin the UI. Only one effect can be shown
    /// at a time.
    ContinuousDisplayEffect(QueryDelegate<CardId, ContinuousDisplayEffect>),
    /// Queries card status markers, which are visual indications of ongoing
    /// effects.
    CardStatusMarkers(QueryDelegate<CardId, Vec<CardStatusMarker>>),
}

impl Delegate {
    pub fn kind(&self) -> DelegateKind {
        self.into()
    }
}

impl fmt::Debug for Delegate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Delegate::{:?}", DelegateKind::from(self))
    }
}

/// Contains the state needed to invoke a delegate within the context of a
/// specific game.
#[derive(Clone, Debug)]
pub struct DelegateContext {
    pub delegate: Delegate,
    pub scope: Scope,
}

/// Caches delegates in a given game for faster lookup
#[derive(Clone, Debug, Default)]
pub struct DelegateCache {
    pub lookup: HashMap<DelegateKind, Vec<DelegateContext>>,
}

impl DelegateCache {
    pub fn delegate_count(&self, kind: DelegateKind) -> usize {
        self.lookup.get(&kind).map_or(0, Vec::len)
    }

    /// Gets the [DelegateContext] for a given [DelegateKind] and index.
    ///
    /// Panics if no such delegate exists.
    pub fn get(&self, kind: DelegateKind, index: usize) -> &DelegateContext {
        &self.lookup.get(&kind).expect("Delegate")[index]
    }
}

/// Functions implemented by an Event struct, automatically implemented by
/// deriving [DelegateEnum]
pub trait EventData<T: fmt::Debug>: fmt::Debug {
    /// Get the underlying data for this event
    fn data(&self) -> &T;

    fn kind(&self) -> DelegateKind;

    /// Return the wrapped [EventDelegate] if the provided [Delegate] is of the
    /// matching type.
    fn extract(delegate: &Delegate) -> Option<&EventDelegate<T>>;
}

/// Functions implemented by a Query struct, automatically implemented by
/// deriving [DelegateEnum]
pub trait QueryData<TData: fmt::Debug, TResult: fmt::Debug>: fmt::Debug {
    /// Get the underlying data for this query
    fn data(&self) -> &TData;

    fn kind(&self) -> DelegateKind;

    /// Return the wrapped [QueryDelegate] if the provided [Delegate] is of the
    /// matching type.
    fn extract(delegate: &Delegate) -> Option<&QueryDelegate<TData, TResult>>;
}
