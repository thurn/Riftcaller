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

//! Core data structures for tracking the state of an ongoing game.

#![allow(clippy::use_self)] // Required to use EnumKind

use std::collections::HashMap;

use anyhow::Result;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use with_error::{fail, WithError};

use crate::card_state::{AbilityState, CardPosition, CardState};
use crate::deck::Deck;
use crate::delegates::DelegateCache;
use crate::game_actions::GamePrompt;
use crate::player_name::PlayerId;
use crate::primitives::{
    AbilityId, ActionCount, CardId, GameId, HasAbilityId, ItemLocation, ManaValue, PointsValue,
    RaidId, RoomId, RoomLocation, School, Side, TurnNumber,
};
use crate::raid_data::RaidData;
use crate::tutorial_data::GameTutorialState;
use crate::updates::{GameUpdate, UpdateStep, UpdateTracker, Updates};

/// Mana to be spent only during the `raid_id` raid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificRaidMana {
    pub raid_id: RaidId,
    pub mana: ManaValue,
}

/// Stores a player's mana, both a general-purpose pool and various
/// restricted-purpose pools.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManaState {
    /// General mana, can be used for any purpose.
    pub base_mana: ManaValue,

    /// Mana which can be used only during a specific raid.
    pub specific_raid_mana: Option<SpecificRaidMana>,
}

/// State of a player within a game, containing their score and available
/// resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayerData {
    pub id: PlayerId,
    pub schools: Vec<School>,
    pub mana_state: ManaState,
    pub actions: ActionCount,
    pub score: PointsValue,

    /// A queue of choices this player is facing related to game choices.
    ///
    /// Choices are resolved in a first-in, first-out manner, i.e. the prompt at
    /// index 0 is presented to the user first. All prompts here take precedence
    /// over choices deriving from game rules such as raid actions.
    pub prompt_queue: Vec<GamePrompt>,
}

impl GamePlayerData {
    /// Create an empty player state.
    pub fn new(id: PlayerId, schools: Vec<School>) -> Self {
        Self {
            id,
            schools,
            mana_state: ManaState::default(),
            actions: 0,
            score: 0,
            prompt_queue: vec![],
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
    /// The mulligan decision for the Overlord player, or None if no decision
    /// has been made.
    pub overlord: Option<MulliganDecision>,
    /// The mulligan decision for the Champion player, or None if no decision
    /// has been made.
    pub champion: Option<MulliganDecision>,
}

impl MulliganData {
    pub fn decision(&self, side: Side) -> Option<&MulliganDecision> {
        match side {
            Side::Overlord => &self.overlord,
            Side::Champion => &self.champion,
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
    /// Counter to create unique IDs for raids within this game
    pub next_raid_id: u32,
    /// Position within the game tutorial, if any
    pub tutorial_state: GameTutorialState,
    /// Game options at creation
    pub config: GameConfiguration,
}

/// State for an individual room
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoomState {
    /// When was a raid last initiated for this room?
    pub last_raided: Option<TurnData>,
}

/// Records a single event which happened during this game.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HistoryEvent {
    /// A card was played from hand via the 'play card' game action
    PlayedCard(CardId),
    /// A raid was started, either via a card effect or the explicit game action
    RaidBegan(RoomId),
    /// A raid ended in success.
    RaidSuccess(RoomId),
    /// A raid ended in failure.
    RaidFailure(RoomId),
}

/// Records a history of events which have happened during this game.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub turn: TurnData,
    pub event: HistoryEvent,
}

/// Stores the primary state for an ongoing game
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Unique identifier for this game
    pub id: GameId,
    /// General game state & configuration
    pub info: GameInfo,
    /// State of the ongoing raid in this game, if any
    pub raid: Option<RaidData>,
    /// Used to track changes to game state in order to update the client. See
    /// [UpdateTracker] for more information.
    #[serde(skip)]
    pub updates: UpdateTracker,
    /// Cards for the overlord player. In general, code should use one of the
    /// helper methods below instead of accessing this directly.
    pub overlord_cards: Vec<CardState>,
    /// Cards for the champion player. In general, code should use one of the
    /// helper methods below instead of accessing this directly.
    pub champion_cards: Vec<CardState>,
    /// State for the overlord player
    pub overlord: GamePlayerData,
    /// State for the champion player
    pub champion: GamePlayerData,
    /// State for abilities of cards in this game
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub ability_state: HashMap<AbilityId, AbilityState>,
    /// History of game actions which have happened during this game. This is
    /// always updated *after* applying an action, i.e. it will typically not
    /// include the action currently being resolved.
    pub history: Vec<HistoryEntry>,
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
}

impl GameState {
    /// Creates a new game with the provided [GameId] and decks for both
    /// players in the [GamePhase::ResolveMulligans] phase.
    ///
    /// Does *not* handle dealing opening hands, prompting for mulligan
    /// decisions, assigning starting mana, etc.
    pub fn new(
        id: GameId,
        overlord: PlayerId,
        overlord_deck: Deck,
        champion: PlayerId,
        champion_deck: Deck,
        config: GameConfiguration,
    ) -> Self {
        Self {
            id,
            info: GameInfo {
                phase: GamePhase::ResolveMulligans(MulliganData::default()),
                turn: TurnData { side: Side::Overlord, turn_number: 0 },
                turn_state: TurnState::Active,
                next_raid_id: 1,
                tutorial_state: GameTutorialState::default(),
                config,
            },
            raid: None,
            overlord_cards: Self::make_deck(&overlord_deck, Side::Overlord),
            champion_cards: Self::make_deck(&champion_deck, Side::Champion),
            overlord: GamePlayerData::new(overlord, overlord_deck.schools),
            champion: GamePlayerData::new(champion, champion_deck.schools),
            ability_state: HashMap::new(),
            history: vec![],
            updates: UpdateTracker::new(if config.simulation {
                Updates::Ignore
            } else {
                Updates::Push
            }),
            next_sorting_key: 1,
            delegate_cache: DelegateCache::default(),
            rng: if config.deterministic {
                Some(Xoshiro256StarStar::seed_from_u64(314159265358979323))
            } else {
                None
            },
        }
    }

    pub fn record_update(&mut self, update: impl FnOnce() -> GameUpdate) {
        if self.updates.state == Updates::Push {
            // Snapshot current game state, omit things that aren't important for display
            // logic.
            let clone = Self {
                id: self.id,
                info: self.info.clone(),
                raid: self.raid.clone(),
                updates: UpdateTracker::new(Updates::Ignore),
                overlord_cards: self.overlord_cards.clone(),
                champion_cards: self.champion_cards.clone(),
                overlord: self.overlord.clone(),
                champion: self.champion.clone(),
                ability_state: self.ability_state.clone(),
                history: self.history.clone(),
                next_sorting_key: self.next_sorting_key,
                rng: None,
                delegate_cache: DelegateCache::default(),
            };

            self.updates.steps.push(UpdateStep { snapshot: clone, update: update() });
        }
    }

    /// Makes a clone of the game state without including the [UpdateTracker]
    /// data.
    pub fn clone_without_updates(&self) -> Self {
        Self {
            id: self.id,
            info: self.info.clone(),
            raid: self.raid.clone(),
            updates: UpdateTracker::default(),
            overlord_cards: self.overlord_cards.clone(),
            champion_cards: self.champion_cards.clone(),
            overlord: self.overlord.clone(),
            champion: self.champion.clone(),
            ability_state: self.ability_state.clone(),
            history: self.history.clone(),
            next_sorting_key: self.next_sorting_key,
            rng: self.rng.clone(),
            delegate_cache: self.delegate_cache.clone(),
        }
    }

    /// Look up [CardState] for a card. Panics if this card is not present in
    /// the game.
    pub fn card(&self, card_id: CardId) -> &CardState {
        &self.cards(card_id.side)[card_id.index]
    }

    /// Mutable version of [Self::card]
    pub fn card_mut(&mut self, card_id: CardId) -> &mut CardState {
        &mut self.cards_mut(card_id.side)[card_id.index]
    }

    /// Cards for a player, in an unspecified order
    pub fn cards(&self, side: Side) -> &Vec<CardState> {
        match side {
            Side::Overlord => &self.overlord_cards,
            Side::Champion => &self.champion_cards,
        }
    }

    /// Mutable version of [Self::cards]
    pub fn cards_mut(&mut self, side: Side) -> &mut Vec<CardState> {
        match side {
            Side::Overlord => &mut self.overlord_cards,
            Side::Champion => &mut self.champion_cards,
        }
    }

    /// State for the players in the game
    pub fn player(&self, side: Side) -> &GamePlayerData {
        match side {
            Side::Overlord => &self.overlord,
            Side::Champion => &self.champion,
        }
    }

    /// Mutable version of [Self::player]
    pub fn player_mut(&mut self, side: Side) -> &mut GamePlayerData {
        match side {
            Side::Overlord => &mut self.overlord,
            Side::Champion => &mut self.champion,
        }
    }

    /// Returns the [Side] the indicated player is representing in this game
    pub fn player_side(&self, player_id: PlayerId) -> Result<Side> {
        if player_id == self.champion.id {
            Ok(Side::Champion)
        } else if player_id == self.overlord.id {
            Ok(Side::Overlord)
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

    /// Returns Overlord cards defending a given room in an unspecified order
    pub fn defenders_unordered(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Overlord, CardPosition::Room(room_id, RoomLocation::Defender))
    }

    /// Overlord cards defending a given room, in sorting-key order (higher
    /// array indices are closer to the front of the room).
    pub fn defender_list(&self, room_id: RoomId) -> Vec<CardId> {
        self.card_list_for_position(
            Side::Overlord,
            CardPosition::Room(room_id, RoomLocation::Defender),
        )
    }

    /// Overlord cards in a given room (not defenders), in an unspecified order
    pub fn occupants(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Overlord, CardPosition::Room(room_id, RoomLocation::Occupant))
    }

    /// All overlord cards which occupy rooms (not defenders), in an unspecified
    /// order
    pub fn occupants_in_all_rooms(&self) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Overlord)
            .iter()
            .filter(move |c| matches!(c.position(), CardPosition::Room(_, RoomLocation::Occupant)))
    }

    /// All Overlord cards located within a given room, defenders and occupants,
    /// in an unspecified order.
    pub fn defenders_and_occupants(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Overlord)
            .iter()
            .filter(move |c| matches!(c.position(), CardPosition::Room(r, _) if r == room_id))
    }

    /// All cards in play for the given side.
    pub fn all_permanents(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(move |c| c.position().in_play())
    }

    /// All overlord defenders in play, whether face-up or face-down.
    pub fn minions(&self) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Overlord)
            .iter()
            .filter(move |c| matches!(c.position(), CardPosition::Room(_, RoomLocation::Defender)))
    }

    /// Champion cards which have been played as artifacts, in an unspecified
    /// order.
    pub fn artifacts(&self) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Champion, CardPosition::ArenaItem(ItemLocation::Artifacts))
    }

    /// Champion cards which have been played as evocations, in an unspecified
    /// order.
    pub fn evocations(&self) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Champion, CardPosition::ArenaItem(ItemLocation::Evocations))
    }

    /// Champion cards which have been played as allies, in an unspecified
    /// order.
    pub fn allies(&self) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Champion, CardPosition::ArenaItem(ItemLocation::Allies))
    }

    /// All global game modifier cards, in an unspecified order
    pub fn game_modifiers(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(side, CardPosition::GameModifier)
    }

    /// All Card IDs present in this game.
    ///
    /// Overlord cards in an unspecified order followed by Champion cards in
    /// an unspecified order.
    pub fn all_card_ids(&self) -> impl Iterator<Item = CardId> {
        (0..self.overlord_cards.len())
            .map(|index| CardId::new(Side::Overlord, index))
            .chain((0..self.champion_cards.len()).map(|index| CardId::new(Side::Champion, index)))
    }

    /// All cards in this game.
    ///
    /// Overlord cards in an unspecified order followed by Champion cards in
    /// an unspecified order.
    pub fn all_cards(&self) -> impl Iterator<Item = &CardState> {
        self.overlord_cards.iter().chain(self.champion_cards.iter())
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
    /// a raid. Returns an error if there is no active raid or no defender is
    /// being encountered.
    pub fn raid_defender(&self) -> Result<CardId> {
        Ok(*self
            .defender_list(self.raid()?.target)
            .get(self.raid()?.encounter.with_error(|| "Expected active encounter")?)
            .with_error(|| "Defender Not Found")?)
    }

    /// Retrieves the [AbilityState] for an [AbilityId]
    pub fn ability_state(&self, ability_id: impl HasAbilityId) -> Option<&AbilityState> {
        self.ability_state.get(&ability_id.ability_id())
    }

    /// Returns a mutable [AbilityState] for an [AbilityId], creating a new one
    /// if one has not previously been set
    pub fn ability_state_mut(&mut self, ability_id: impl HasAbilityId) -> &mut AbilityState {
        self.ability_state.entry(ability_id.ability_id()).or_insert_with(AbilityState::default)
    }

    /// Adds a [HistoryEvent] for the current turn.
    pub fn add_history(&mut self, event: HistoryEvent) {
        self.history.push(HistoryEntry { turn: self.info.turn, event })
    }

    /// Create card states for a deck
    fn make_deck(deck: &Deck, side: Side) -> Vec<CardState> {
        let mut result = vec![];

        for (i, riftcaller) in deck.riftcallers.iter().enumerate() {
            // Put all riftcaller cards into play face up
            let mut card = CardState::new(CardId::new(side, i), *riftcaller);
            card.set_position_internal(i as u32, CardPosition::Riftcaller(side));
            card.turn_face_up();
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::card_name::CardName;
    use crate::player_name::AIPlayer;

    #[test]
    fn insert_at_index() {
        let (abyssal, infernal, mortal) = (
            CardId::new(Side::Overlord, 0),
            CardId::new(Side::Overlord, 1),
            CardId::new(Side::Overlord, 2),
        );
        let mut g = test_game(
            vec![
                CardName::TestAbyssalMinion,
                CardName::TestInfernalMinion,
                CardName::TestMortalMinion,
            ],
            vec![],
        );

        fn hand(g: &GameState) -> Vec<CardId> {
            g.card_list_for_position(Side::Overlord, CardPosition::Hand(Side::Overlord))
        }

        fn hand_key_count(g: &GameState) -> usize {
            hand(g).iter().map(|id| g.card(*id).sorting_key).collect::<HashSet<_>>().len()
        }

        g.move_card_internal(abyssal, CardPosition::Hand(Side::Overlord));
        g.move_card_internal(infernal, CardPosition::Hand(Side::Overlord));
        g.move_card_internal(mortal, CardPosition::Hand(Side::Overlord));
        assert_eq!(3, hand_key_count(&g));
        assert_eq!(vec![abyssal, infernal, mortal], hand(&g));

        g.move_card_to_index(mortal, 0);
        assert_eq!(3, hand_key_count(&g));
        assert_eq!(vec![mortal, abyssal, infernal], hand(&g));

        g.move_card_to_index(abyssal, 1);
        assert_eq!(3, hand_key_count(&g));
        assert_eq!(vec![mortal, abyssal, infernal], hand(&g));

        g.move_card_to_index(abyssal, 2);
        assert_eq!(3, hand_key_count(&g));
        assert_eq!(vec![mortal, infernal, abyssal], hand(&g));

        g.move_card_to_index(abyssal, usize::MAX);
        assert_eq!(3, hand_key_count(&g));
        assert_eq!(vec![mortal, infernal, abyssal], hand(&g));
    }

    fn test_game(overlord: Vec<CardName>, champion: Vec<CardName>) -> GameState {
        GameState::new(
            GameId::new(0),
            PlayerId::AI(AIPlayer::NoAction),
            Deck {
                side: Side::Overlord,
                schools: vec![School::Law],
                riftcallers: vec![],
                cards: overlord.into_iter().map(|name| (name, 1)).collect(),
            },
            PlayerId::AI(AIPlayer::NoAction),
            Deck {
                side: Side::Champion,
                schools: vec![School::Law],
                riftcallers: vec![],
                cards: champion.into_iter().map(|name| (name, 1)).collect(),
            },
            GameConfiguration { deterministic: true, ..GameConfiguration::default() },
        )
    }
}
