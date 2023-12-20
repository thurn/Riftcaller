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

//! Defines the state of cards during an ongoing game.

#![allow(clippy::use_self)] // Required to use EnumKind

use std::cmp::Ordering;

use core_data::game_primitives::{
    CardId, CardPlayId, ItemLocation, ManaValue, PowerChargeValue, ProgressValue, RoomId,
    RoomLocation, Side,
};
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use crate::card_name::CardVariant;
use crate::custom_card_state::CustomCardStateList;
use crate::game_actions::CardTarget;
#[allow(unused_imports)] // Used in Rustdocs
use crate::history_data::HistoryEvent;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub struct BanishedByCard {
    pub source: CardId,
    pub play_id: CardPlayId,
}

/// Identifies the location of a card during an active game
#[derive(
    PartialEq, Eq, Hash, Debug, Copy, Clone, EnumKind, Serialize, Deserialize, Ord, PartialOrd,
)]
#[enum_kind(CardPositionKind)]
pub enum CardPosition {
    /// An unspecified random position within a user's deck. The default
    /// position of all cards when a new game is started.
    DeckUnknown(Side),
    /// A card which is known to at least one player to be on the top of a deck
    DeckTop(Side),
    Hand(Side),
    Room(CardPlayId, RoomId, RoomLocation),
    ArenaItem(CardPlayId, ItemLocation),
    DiscardPile(Side),
    /// A card has been scored and is currently resolving its scoring effects
    /// before moving to a score pile.
    Scoring,
    /// Card is in the [Side] player's score pile
    Scored(Side),
    /// A card has been played by the [Side] player and is in the process of
    /// resolving with the provided target
    Played(CardPlayId, Side, CardTarget),
    /// A riftcaller or chapter card owned by a player in the game.
    Identity(Side),
    /// Global modifier cards which change the rules of the game
    GameModifier,
    /// A sigil card which starts the game in play and provides ongoing effects
    Sigil(Side),
    /// A card has been banished, either temporarily or permanently removed from
    /// the game. A [BanishedByCard] can be provided to record data around how
    /// the card was banished.
    Banished(Option<BanishedByCard>),
}

impl CardPosition {
    /// Returns the [CardPositionKind] for this card
    pub fn kind(&self) -> CardPositionKind {
        self.into()
    }

    /// Returns the [CardPlayId] if this position currently contains one.
    ///
    /// Prefer to use [CardState]'s `last_card_play_id` instead of referencing
    /// this directly.
    pub fn card_play_id(&self) -> Option<CardPlayId> {
        match self {
            CardPosition::Room(id, ..)
            | CardPosition::ArenaItem(id, ..)
            | CardPosition::Played(id, ..) => Some(*id),
            _ => None,
        }
    }

    /// Returns true if this card is an identity, is in a room, or has been
    /// played as an item.
    pub fn in_play(&self) -> bool {
        matches!(
            self.kind(),
            CardPositionKind::Identity | CardPositionKind::Room | CardPositionKind::ArenaItem
        )
    }

    /// Returns true if this card is in a room
    pub fn in_room(&self) -> bool {
        self.kind() == CardPositionKind::Room
    }

    /// Returns true if this card is in a specific room
    pub fn in_specified_room(&self, room_id: RoomId) -> bool {
        matches!(self, CardPosition::Room(_, r, _) if *r == room_id)
    }

    /// Returns true if this card is in a user's hand
    pub fn in_hand(&self) -> bool {
        self.kind() == CardPositionKind::Hand
    }

    // True if a card is currently shuffled into a deck
    pub fn shuffled_into_deck(&self) -> bool {
        self.kind() == CardPositionKind::DeckUnknown
    }

    /// Returns true if this card is in a known or unknown deck position
    pub fn in_deck(&self) -> bool {
        matches!(self.kind(), CardPositionKind::DeckUnknown | CardPositionKind::DeckTop)
    }

    /// Returns true if this card is in an unknown position in the user's deck.
    pub fn in_deck_unknown(&self) -> bool {
        matches!(self.kind(), CardPositionKind::DeckUnknown)
    }

    /// Returns true if this card is in a user's discard pile
    pub fn in_discard_pile(&self) -> bool {
        self.kind() == CardPositionKind::DiscardPile
    }

    /// True if this card is in a room occupant position
    pub fn is_occupant(&self) -> bool {
        matches!(
            self,
            CardPosition::Room(_, _, location)
            if *location == RoomLocation::Occupant
        )
    }

    /// True if this card is an occupant of the specified room    
    pub fn is_occupant_of(&self, room_id: RoomId) -> bool {
        matches!(
            self,
            CardPosition::Room(_, r, location)
            if *location == RoomLocation::Occupant && *r == room_id
        )
    }

    /// True if this card is in a room defender position
    pub fn is_defender(&self) -> bool {
        matches!(
            self,
            CardPosition::Room(_, _, location)
            if *location == RoomLocation::Defender
        )
    }

    /// True if this card is a defender of the specified room
    pub fn is_defender_of(&self, room_id: RoomId) -> bool {
        matches!(
            self,
            CardPosition::Room(_, r, location)
            if *location == RoomLocation::Defender && *r == room_id
        )
    }

    /// Returns true if this card is in a user's score pile
    pub fn in_score_pile(&self) -> bool {
        self.kind() == CardPositionKind::Scored
    }

    /// Returns true if this card has been banished
    pub fn is_banished(&self) -> bool {
        self.kind() == CardPositionKind::Banished
    }
}

/// A counter which can be placed on a card to track some kind of numeric state
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CardCounter {
    Progress,
    StoredMana,
    PowerCharges,
}

/// Optional card state, properties which have a default.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardData {
    /// How many times has this card been progressed?
    progress: ProgressValue,
    /// How much mana is stored in this card?
    stored_mana: ManaValue,
    /// Number of power charges on this card.
    power_charges: PowerChargeValue,
    /// Is this card face-up?
    is_face_up: bool,
    /// Is this card visible to the [CardId.side] user?
    visible_to_owner: bool,
    /// Is this card visible to opponent of the [CardId.side] user?
    visible_to_opponent: bool,
}

/// Stores the state of a Card during an ongoing game. The game rules for a
/// card are not part of its state, see [crate::card_definition::CardDefinition]
/// for that.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct CardState {
    /// ID for this card.
    pub id: CardId,
    /// Identifies this card within the game rules, using its card name and
    /// other properties.
    ///
    /// Used to look up this card's definition.
    pub variant: CardVariant,
    /// Optional state for this card
    data: CardData,
    /// Opaque value identifying this card's sort order within its CardPosition.
    /// Higher sorting keys are closer to the 'top' or 'front' of the position.
    pub sorting_key: u32,
    position: CardPosition,
    /// A unique identifier for an instance of this card being played.
    ///
    /// This stores the last-known [CardPlayId]. If this card is currently in
    /// play, this is the ID which was assigned when the card was played. If the
    /// card was moved from play to a non-play zone, this contains the most
    /// recent in-play ID. Otherwise, it contains None.
    pub last_card_play_id: Option<CardPlayId>,
    /// Stores custom state entries for this card.
    ///
    /// See [CustomCardStateList].
    pub custom_state: CustomCardStateList,
}

/// Helper trait to build a vector of card IDs from a card state iterator.
pub trait CardIdsExt {
    fn card_ids(self) -> Vec<CardId>;
}

impl<'a, T> CardIdsExt for T
where
    T: Iterator<Item = &'a CardState>,
{
    fn card_ids(self) -> Vec<CardId> {
        self.map(|c| c.id).collect()
    }
}

impl CardState {
    /// Creates a new card state
    pub fn new(id: CardId, variant: CardVariant) -> Self {
        Self {
            id,
            variant,
            position: CardPosition::DeckUnknown(id.side),
            sorting_key: 0,
            data: CardData {
                visible_to_owner: false,
                visible_to_opponent: false,
                is_face_up: false,
                ..CardData::default()
            },
            last_card_play_id: None,
            custom_state: CustomCardStateList::default(),
        }
    }

    /// Creates a new [CardState] with a given position and face-up state.
    pub fn new_with_position(
        id: CardId,
        name: CardVariant,
        position: CardPosition,
        sorting_key: u32,
        is_face_up: bool,
    ) -> Self {
        Self {
            id,
            variant: name,
            position,
            sorting_key,
            data: CardData {
                visible_to_owner: is_face_up,
                visible_to_opponent: is_face_up,
                is_face_up,
                ..CardData::default()
            },
            last_card_play_id: None,
            custom_state: CustomCardStateList::default(),
        }
    }

    /// Retrieves the current value for a [CardCounter], or 0 if this card is
    /// not in play.
    pub fn counters(&self, counter: CardCounter) -> u32 {
        if self.position.in_play() {
            self.last_known_counters(counter)
        } else {
            0
        }
    }

    /// Retrieves the last known value for a [CardCounter], returning a value
    /// even if this card is not in play.
    pub fn last_known_counters(&self, counter: CardCounter) -> u32 {
        match counter {
            CardCounter::Progress => self.data.progress,
            CardCounter::StoredMana => self.data.stored_mana,
            CardCounter::PowerCharges => self.data.power_charges,
        }
    }

    /// Adds a `amount` [CardCounter]s to this card
    pub fn add_counters(&mut self, counter: CardCounter, amount: u32) {
        *self.counters_mut(counter) = self.counters(counter) + amount;
    }

    /// Removes *up to* `amount` [CardCounter]s from this card
    pub fn remove_counters_saturating(&mut self, counter: CardCounter, amount: u32) {
        *self.counters_mut(counter) = self.counters(counter).saturating_sub(amount);
    }

    /// Sets the current `amount` of [CardCounter]s on this card
    pub fn set_counters(&mut self, counter: CardCounter, amount: u32) {
        *self.counters_mut(counter) = amount;
    }

    /// Clears all stored counters for this card.
    pub fn clear_counters(&mut self) {
        self.data.progress = 0;
        self.data.stored_mana = 0;
        self.data.power_charges = 0;
    }

    pub fn side(&self) -> Side {
        self.id.side
    }

    /// Where this card is located in the game.
    pub fn position(&self) -> CardPosition {
        self.position
    }

    /// Sets the position of this card. Please use `mutations::move_card`
    /// instead of invoking this directly.
    pub fn set_position_internal(&mut self, sorting_key: u32, position: CardPosition) {
        self.sorting_key = sorting_key;
        self.position = position;
    }

    /// Whether this card is in the 'face up' state.
    pub fn is_face_up(&self) -> bool {
        self.data.is_face_up
    }

    /// Whether this card is in the 'face down' state.
    pub fn is_face_down(&self) -> bool {
        !self.data.is_face_up
    }

    /// Returns true if this card is currently visible to the indicated user
    ///
    /// Note that this is not the same as [Self::is_face_up], both players may
    /// know a card without it being the the 'face up' state.
    pub fn is_visible_to(&self, side: Side) -> bool {
        if self.id.side == side {
            self.data.visible_to_owner
        } else {
            self.data.visible_to_opponent
        }
    }

    /// Returns true if the provided [CardId] is the last-selected target for
    /// the current instance of this card being played.
    pub fn is_last_target(&self, card_id: CardId) -> bool {
        let Some(card_play_id) = self.last_card_play_id else {
            return false;
        };
        self.custom_state.targets(card_play_id).last() == Some(card_id)
    }

    /// Change a card to the 'face up' state and makes the card revealed to both
    /// players.
    pub fn internal_turn_face_up(&mut self) {
        self.data.is_face_up = true;
        self.internal_set_visible_to(Side::Covenant, true);
        self.internal_set_visible_to(Side::Riftcaller, true);
    }

    /// Change a card to the 'face down' state, but does *not* change its
    /// revealed state for either player.
    pub fn internal_turn_face_down(&mut self) {
        self.data.is_face_up = false;
    }

    /// Updates the 'visible' state of a card to be visible to the indicated
    /// `side` player. Note that this is *not* the same as turning a card
    /// face-up, a card can be visible to both players without being
    /// face-up
    pub fn internal_set_visible_to(&mut self, side: Side, revealed: bool) {
        if self.id.side == side {
            self.data.visible_to_owner = revealed
        } else {
            self.data.visible_to_opponent = revealed
        }
    }

    fn counters_mut(&mut self, counter: CardCounter) -> &mut u32 {
        match counter {
            CardCounter::Progress => &mut self.data.progress,
            CardCounter::StoredMana => &mut self.data.stored_mana,
            CardCounter::PowerCharges => &mut self.data.power_charges,
        }
    }
}

impl PartialOrd<Self> for CardState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CardState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sorting_key.cmp(&other.sorting_key)
    }
}
