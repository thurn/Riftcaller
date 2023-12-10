// Copyright © Riftcaller 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Core data structures for tracking the state of an ongoing game.

#![allow(clippy::use_self)] // Required to use EnumKind

use anyhow::Result;
use core_data::game_primitives::{
    ActionCount, CardId, CurseCount, GameId, HasCardId, ItemLocation, LeylineCount, ManaValue,
    PointsValue, RaidId, RoomId, RoomLocation, School, Side, TurnNumber, WoundCount,
};
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use with_error::{fail, WithError};

use crate::animation_tracker::{AnimationState, AnimationStep, AnimationTracker, GameAnimation};
use crate::card_state::{CardPosition, CardState};
use crate::deck::Deck;
use crate::delegate_data::DelegateCache;
use crate::history_data::{GameHistory, HistoryCounters, HistoryEvent};
use crate::player_name::PlayerId;
use crate::prompt_data::PromptStack;
use crate::raid_data::RaidData;
use crate::state_machine_data::StateMachines;
use crate::tutorial_data::GameTutorialState;
use crate::undo_tracker::UndoTracker;

/// Stores a player's mana, both a general-purpose pool and various
/// restricted-purpose pools.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManaState {
    /// General mana, can be used for any purpose.
    pub base_mana: ManaValue,
    /// Mana which can be used only during a specific raid.
    pub raid_mana: Option<(RaidId, ManaValue)>,
}

/// Stores state for a player's curses. Please use the `curses` module instead
/// of interacting directly with this data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CurseState {
    /// General curse count.
    pub base_curses: CurseCount,
    /// Curses given for the duration of a specific turn.
    pub turn_curses: Option<(TurnData, CurseCount)>,
}

/// State of a player within a game, containing their score and available
/// resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayerData {
    pub id: PlayerId,
    pub mana_state: ManaState,
    pub actions: ActionCount,
    pub curse_state: CurseState,
    pub wounds: WoundCount,
    pub leylines: LeylineCount,
    pub bonus_points: PointsValue,

    /// Schools for this player's deck, mostly used to determine which card
    /// back & card frame assets get shown.
    pub schools: Vec<School>,

    pub prompts: PromptStack,

    /// Storage area for cards this player has selected. Sometimes we show
    /// multi-step prompt_ui like "select 2 artifacts" and need a place to store
    /// the state while that is evaluating, and this seems like a fine enough
    /// place to put it.
    pub prompt_selected_cards: Vec<CardId>,
}

impl GamePlayerData {
    /// Create an empty player state.
    pub fn new(id: PlayerId, schools: Vec<School>) -> Self {
        Self {
            id,
            mana_state: ManaState::default(),
            actions: 0,
            curse_state: CurseState::default(),
            wounds: 0,
            leylines: 0,
            bonus_points: 0,
            schools,
            prompts: PromptStack::default(),
            prompt_selected_cards: vec![],
        }
    }
}

/// Some card abilities completely change the state of a Raid, for example to
/// target a different room or encounter a different minion. Setting a jump
/// request on the [RaidData] asks the raid system to execute the related
/// transformation when possible.
///
/// This always happens *after* processing the current raid state and only if
/// the raid is still active, i.e. it cannot be used to stop the effects of a
/// user action from happening.
///
/// Only one jump request is supported at a time, on a 'last write wins' basis.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RaidJumpRequest {
    EncounterMinion(CardId),
    ChangeTarget(RoomId),

    /// Evades a minion during a raid. Use `mutations::evade_current_minion`
    /// instead of applying this directly.
    EvadeCurrentMinion,
}

/// Describes options for this game & the set of rules it is using.
#[derive(Debug, Clone, Default, Copy, Serialize, Deserialize)]
pub struct GameConfiguration {
    /// If true, all random choices within this game will be made
    /// deterministically using a seeded random number generator. Useful for
    /// e.g. unit tests.
    pub deterministic: bool,
    /// Whether to run in simulation mode and thus disable update tracking
    pub simulation: bool,
    /// Whether to overwrite the normal game behavior with the standard
    /// pre-scripted new player experience.
    pub scripted_tutorial: bool,
}

/// Mulligan decision a player made for their opening hand
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum MulliganDecision {
    /// The player has decided to keep their initial hand of 5 cards
    Keep,
    /// The player has elected to draw a new hand of 5 cards
    Mulligan,
}

/// [MulliganDecision]s for both players.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct MulliganData {
    /// The mulligan decision for the Covenant player, or None if no decision
    /// has been made.
    pub covenant: Option<MulliganDecision>,
    /// The mulligan decision for the Riftcaller player, or None if no decision
    /// has been made.
    pub riftcaller: Option<MulliganDecision>,
}

impl MulliganData {
    pub fn decision(&self, side: Side) -> Option<&MulliganDecision> {
        match side {
            Side::Covenant => &self.covenant,
            Side::Riftcaller => &self.riftcaller,
        }
        .as_ref()
    }
}

/// Identifies a turn within the game.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TurnData {
    /// Player whose turn it was.
    pub side: Side,
    /// Turn number for that player
    pub turn_number: TurnNumber,
}

/// High level status of a game
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum GamePhase {
    ResolveMulligans(MulliganData),
    Play,
    GameOver { winner: Side },
}

impl GamePhase {
    /// Returns true if the current game phase is [GamePhase::Play].
    pub fn is_playing(&self) -> bool {
        *self == GamePhase::Play
    }
}

/// Information about the state of the current turn.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TurnState {
    /// The active player is playing their turn
    Active,
    /// The active player has ended their turn, their opponent can now elect to
    /// use effects or start their own turn.
    Ended,
}

/// Information about the overall game, including whose turn it is and whether a
/// raid is active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    /// Current [GamePhase].
    pub phase: GamePhase,
    /// Identifies current game turn
    pub turn: TurnData,
    /// State of the current turn
    pub turn_state: TurnState,
    /// Position within the game tutorial, if any
    pub tutorial_state: GameTutorialState,
    /// Game options at creation
    pub config: GameConfiguration,
    /// Counter to create unique IDs for states within this game
    next_event_id: u32,
}

impl GameInfo {
    /// Counter to create unique IDs for states within this game
    pub fn next_event_id(&mut self) -> u32 {
        let result = self.next_event_id;
        self.next_event_id += 1;
        result
    }
}

/// State for an individual room
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoomState {
    /// When was a raid last initiated for this room?
    pub last_raided: Option<TurnData>,
}

/// Stores the primary state for an ongoing game
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Unique identifier for this game
    pub id: GameId,
    /// General game state & configuration
    pub info: GameInfo,
    /// Collection of state machines for handling resolution of multi-step game
    /// updates.
    pub state_machines: StateMachines,
    /// State of the ongoing raid in this game, if any
    pub raid: Option<RaidData>,
    /// Used to track changes to game state in order to update the client. See
    /// [AnimationTracker] for more information.
    #[serde(skip)]
    pub animations: AnimationTracker,
    /// Cards for the covenant player. In general, code should use one of the
    /// helper methods below instead of accessing this directly.
    pub covenant_cards: Vec<CardState>,
    /// Cards for the riftcaller player. In general, code should use one of the
    /// helper methods below instead of accessing this directly.
    pub riftcaller_cards: Vec<CardState>,
    /// State for the covenant player
    pub covenant: GamePlayerData,
    /// State for the riftcaller player
    pub riftcaller: GamePlayerData,
    ///  History of events which have happened during this game. See
    /// [GameHistory].
    pub history: GameHistory,
    /// Next sorting key to use for card moves. Automatically updated by
    /// [Self::next_sorting_key] and [Self::move_card_internal].
    next_sorting_key: u32,
    /// Optionally, a random number generator for this game to use. This
    /// generator is serializable, so the state will be deterministic even
    /// across different sessions. If not specified, `rand::thread_rng()` is
    /// used instead and behavior is not deterministic.
    pub rng: Option<Xoshiro256StarStar>,
    /// Optional lookup table for delegates present on cards in this game in
    /// order to improve performance
    #[serde(skip)]
    pub delegate_cache: DelegateCache,
    /// Handles state tracking for the 'undo' action.
    pub undo_tracker: Option<UndoTracker>,
}

impl GameState {
    /// Creates a new game with the provided [GameId] and decks for both
    /// players in the [GamePhase::ResolveMulligans] phase.
    ///
    /// Does *not* handle dealing opening hands, prompting for mulligan
    /// decisions, assigning starting mana, etc.
    pub fn new(
        id: GameId,
        covenant: PlayerId,
        covenant_deck: Deck,
        riftcaller: PlayerId,
        riftcaller_deck: Deck,
        config: GameConfiguration,
    ) -> Self {
        let turn = TurnData { side: Side::Covenant, turn_number: 0 };
        Self {
            id,
            info: GameInfo {
                phase: GamePhase::ResolveMulligans(MulliganData::default()),
                turn,
                turn_state: TurnState::Active,
                next_event_id: 1,
                tutorial_state: GameTutorialState::default(),
                config,
            },
            state_machines: StateMachines::default(),
            raid: None,
            covenant_cards: Self::make_deck(&covenant_deck, Side::Covenant),
            riftcaller_cards: Self::make_deck(&riftcaller_deck, Side::Riftcaller),
            covenant: GamePlayerData::new(covenant, covenant_deck.schools),
            riftcaller: GamePlayerData::new(riftcaller, riftcaller_deck.schools),
            history: GameHistory::default(),
            animations: AnimationTracker::new(if config.simulation {
                AnimationState::Ignore
            } else {
                AnimationState::Track
            }),
            next_sorting_key: 1,
            delegate_cache: DelegateCache::default(),
            rng: if config.deterministic {
                Some(Xoshiro256StarStar::seed_from_u64(314159265358979323))
            } else {
                None
            },
            undo_tracker: Some(UndoTracker::default()),
        }
    }

    pub fn add_animation(&mut self, update: impl FnOnce() -> GameAnimation) {
        if self.animations.state == AnimationState::Track {
            // Snapshot current game state, omit things that aren't important for display
            // logic.
            let clone = Self {
                id: self.id,
                info: self.info.clone(),
                state_machines: self.state_machines.clone(),
                raid: self.raid.clone(),
                animations: AnimationTracker::new(AnimationState::Ignore),
                covenant_cards: self.covenant_cards.clone(),
                riftcaller_cards: self.riftcaller_cards.clone(),
                covenant: self.covenant.clone(),
                riftcaller: self.riftcaller.clone(),
                history: self.history.clone(),
                next_sorting_key: self.next_sorting_key,
                rng: None,
                delegate_cache: DelegateCache::default(),
                undo_tracker: None,
            };

            self.animations.steps.push(AnimationStep { snapshot: clone, update: update() });
        }
    }

    /// Makes a clone of the game state without including the [AnimationTracker]
    /// or [UndoTracker] data.
    pub fn clone_for_simulation(&self) -> Self {
        Self {
            id: self.id,
            info: self.info.clone(),
            state_machines: self.state_machines.clone(),
            raid: self.raid.clone(),
            animations: AnimationTracker::default(),
            covenant_cards: self.covenant_cards.clone(),
            riftcaller_cards: self.riftcaller_cards.clone(),
            covenant: self.covenant.clone(),
            riftcaller: self.riftcaller.clone(),
            history: self.history.clone(),
            next_sorting_key: self.next_sorting_key,
            rng: self.rng.clone(),
            delegate_cache: self.delegate_cache.clone(),
            undo_tracker: None,
        }
    }

    /// Look up [CardState] for a card. Panics if this card is not present in
    /// the game.
    pub fn card(&self, id: impl HasCardId) -> &CardState {
        let card_id = id.card_id();
        &self.cards(card_id.side)[card_id.index]
    }

    /// Mutable version of [Self::card]
    pub fn card_mut(&mut self, id: impl HasCardId) -> &mut CardState {
        let card_id = id.card_id();
        &mut self.cards_mut(card_id.side)[card_id.index]
    }

    /// Cards for a player, in an unspecified order
    pub fn cards(&self, side: Side) -> &Vec<CardState> {
        match side {
            Side::Covenant => &self.covenant_cards,
            Side::Riftcaller => &self.riftcaller_cards,
        }
    }

    /// Mutable version of [Self::cards]
    pub fn cards_mut(&mut self, side: Side) -> &mut Vec<CardState> {
        match side {
            Side::Covenant => &mut self.covenant_cards,
            Side::Riftcaller => &mut self.riftcaller_cards,
        }
    }

    /// State for the players in the game
    pub fn player(&self, side: Side) -> &GamePlayerData {
        match side {
            Side::Covenant => &self.covenant,
            Side::Riftcaller => &self.riftcaller,
        }
    }

    /// Mutable version of [Self::player]
    pub fn player_mut(&mut self, side: Side) -> &mut GamePlayerData {
        match side {
            Side::Covenant => &mut self.covenant,
            Side::Riftcaller => &mut self.riftcaller,
        }
    }

    /// Returns the [Side] the indicated player is representing in this game
    pub fn player_side(&self, player_id: PlayerId) -> Result<Side> {
        if player_id == self.riftcaller.id {
            Ok(Side::Riftcaller)
        } else if player_id == self.covenant.id {
            Ok(Side::Covenant)
        } else {
            fail!("Player {:?} is not a participant in game {:?}", player_id, self.id)
        }
    }

    /// Returns a monotonically-increasing sorting key for object positions in
    /// this game.
    pub fn next_sorting_key(&mut self) -> u32 {
        let result = self.next_sorting_key;
        self.next_sorting_key += 1;
        result
    }

    /// Moves a card to a new [CardPosition], updating its sorting key.
    ///
    /// Generally use `mutations::move_card` instead of calling this method
    /// directly.
    pub fn move_card_internal(&mut self, card_id: CardId, new_position: CardPosition) {
        let key = self.next_sorting_key();
        self.card_mut(card_id).set_position_internal(key, new_position);
    }

    /// Moves a card to a given `index` location within its [CardPosition],
    /// shifting all elements after it to the right.
    ///
    /// Moves the card to the end of the list if `index` is out of bounds.
    pub fn move_card_to_index(&mut self, card_id: CardId, mut index: usize) {
        let mut cards = self.card_list_for_position(card_id.side, self.card(card_id).position());
        if index > cards.len() - 1 {
            index = cards.len() - 1;
        }

        cards.retain(|id| *id != card_id);
        cards.insert(index, card_id);

        for id in cards {
            self.card_mut(id).sorting_key = self.next_sorting_key();
        }
    }

    /// Cards owned by a given player in a given position, in an unspecified
    /// order
    pub fn cards_in_position(
        &self,
        side: Side,
        position: CardPosition,
    ) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(move |c| c.position() == position)
    }

    pub fn cards_in_position_mut(
        &mut self,
        side: Side,
        position: CardPosition,
    ) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(move |c| c.position() == position)
    }

    /// Cards owned by a player in a given position, in sorting-key order
    pub fn card_list_for_position(&self, side: Side, position: CardPosition) -> Vec<CardId> {
        let mut result = self.cards_in_position(side, position).collect::<Vec<_>>();
        result.sort();
        result.iter().map(|c| c.id).collect()
    }

    pub fn card_list_for_iterator<'a>(
        &self,
        iterator: impl Iterator<Item = &'a CardState>,
    ) -> Vec<CardId> {
        let mut result = iterator.collect::<Vec<_>>();
        result.sort();
        result.iter().map(|c| c.id).collect()
    }

    /// Cards in a player's hand, in an unspecified order
    pub fn hand(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_hand())
    }

    /// Cards in a player's deck, in an unspecified order
    pub fn deck(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_deck())
    }

    /// Cards in a player's discard pile, in an unspecified order
    pub fn discard_pile(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_discard_pile())
    }

    /// Cards (owned by either player) in a player's score area, in an
    /// unspecified order
    pub fn score_area(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Covenant)
            .iter()
            .filter(move |c| c.position() == CardPosition::Scored(side))
            .chain(
                self.cards(Side::Riftcaller)
                    .iter()
                    .filter(move |c| c.position() == CardPosition::Scored(side)),
            )
    }

    /// Returns Covenant cards defending a given room in an unspecified order
    pub fn defenders_unordered(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Covenant).iter().filter(move |c| c.position().is_defender_of(room_id))
    }

    /// Covenant cards defending a given room, in sorting-key order (higher
    /// array indices are closer to the front of the room).
    pub fn defender_list(&self, room_id: RoomId) -> Vec<CardId> {
        self.card_list_for_iterator(
            self.cards(Side::Covenant).iter().filter(move |c| c.position().is_defender_of(room_id)),
        )
    }

    /// Covenant cards in a given room (not defenders), in an unspecified order
    pub fn occupants(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Covenant).iter().filter(move |c| c.position().is_occupant_of(room_id))
    }

    /// All covenant cards which occupy rooms (not defenders), in an unspecified
    /// order
    pub fn occupants_in_all_rooms(&self) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Covenant).iter().filter(move |c| c.position().is_occupant())
    }

    /// All Covenant cards located within a given room, defenders and occupants,
    /// in an unspecified order.
    pub fn defenders_and_occupants(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Covenant)
            .iter()
            .filter(move |c| matches!(c.position(), CardPosition::Room(_, r, _) if r == room_id))
    }

    /// All cards in play for the given side.
    pub fn all_permanents(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(move |c| c.position().in_play())
    }

    /// All covenant defenders in play, whether face-up or face-down.
    pub fn minions(&self) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Covenant).iter().filter(move |c| {
            matches!(c.position(), CardPosition::Room(_, _, RoomLocation::Defender))
        })
    }

    /// Riftcaller cards which have been played as artifacts, in an unspecified
    /// order.
    pub fn artifacts(&self) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Riftcaller).iter().filter(move |c| {
            matches!(c.position(),
                CardPosition::ArenaItem(_, l) if l == ItemLocation::Artifacts)
        })
    }

    /// Riftcaller cards which have been played as evocations, in an unspecified
    /// order.
    pub fn evocations(&self) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Riftcaller).iter().filter(move |c| {
            matches!(c.position(),
                CardPosition::ArenaItem(_, l) if l == ItemLocation::Evocations)
        })
    }

    /// Riftcaller cards which have been played as allies, in an unspecified
    /// order.
    pub fn allies(&self) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Riftcaller).iter().filter(move |c| {
            matches!(c.position(),
                CardPosition::ArenaItem(_, l) if l == ItemLocation::Allies)
        })
    }

    /// All global game modifier cards, in an unspecified order
    pub fn game_modifiers(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(side, CardPosition::GameModifier)
    }

    /// All Card IDs present in this game.
    ///
    /// Covenant cards in an unspecified order followed by Riftcaller cards in
    /// an unspecified order.
    pub fn all_card_ids(&self) -> impl Iterator<Item = CardId> {
        (0..self.covenant_cards.len()).map(|index| CardId::new(Side::Covenant, index)).chain(
            (0..self.riftcaller_cards.len()).map(|index| CardId::new(Side::Riftcaller, index)),
        )
    }

    /// All cards in this game.
    ///
    /// Covenant cards in an unspecified order followed by Riftcaller cards in
    /// an unspecified order.
    pub fn all_cards(&self) -> impl Iterator<Item = &CardState> {
        self.covenant_cards.iter().chain(self.riftcaller_cards.iter())
    }

    /// Returns the [RaidId] of the ongoing raid, if any.
    pub fn raid_id(&self) -> Option<RaidId> {
        self.raid.as_ref().map(|r| r.raid_id)
    }

    /// Helper method to return the current [RaidData] or an error when one is
    /// expected to exist.
    pub fn raid(&self) -> Result<&RaidData> {
        self.raid.as_ref().with_error(|| "Expected Raid")
    }

    /// Mutable version of [Self::raid].
    pub fn raid_mut(&mut self) -> Result<&mut RaidData> {
        self.raid.as_mut().with_error(|| "Expected Raid")
    }

    /// Helper method to return the defender currently being encountered during
    /// a raid. Returns None if there is no active raid or no defender is
    /// being encountered.
    pub fn current_raid_defender(&self) -> Option<CardId> {
        self.defender_list(self.raid.as_ref()?.target)
            .get(self.raid.as_ref()?.encounter)
            .map(|c| *c)
    }

    /// Adds a current [HistoryEvent] for the current turn.
    pub fn add_history_event(&mut self, event: HistoryEvent) {
        self.history.add_event(self.info.turn, event)
    }

    /// Returns a mutable reference to the [HistoryCounters] for the `side`
    /// player in the current turn.
    pub fn current_history_counters(&mut self, side: Side) -> &mut HistoryCounters {
        self.history.counters_for_turn_mut(self.info.turn, side)
    }

    /// Create card states for a deck
    fn make_deck(deck: &Deck, side: Side) -> Vec<CardState> {
        let mut result = vec![];

        for (i, identity) in deck.identities.iter().enumerate() {
            // Put all identity cards into play face up
            let mut card = CardState::new(CardId::new(side, i), *identity);
            card.set_position_internal(i as u32, CardPosition::Identity(side));
            card.internal_turn_face_up();
            result.push(card);
        }

        let offset = result.len();
        result.extend(
            deck.card_names()
                .iter()
                .enumerate()
                .map(move |(index, name)| CardState::new(CardId::new(side, index + offset), *name)),
        );

        result
    }
}
