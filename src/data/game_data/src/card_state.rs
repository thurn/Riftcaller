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

//! Defines the state of cards during an ongoing game.

#![allow(clippy::use_self)] // Required to use EnumKind

use std::cmp::Ordering;

use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use crate::card_name::CardVariant;
use crate::game_actions::CardTarget;
use crate::primitives::{
    CardId, ItemLocation, ManaValue, PowerChargeValue, ProgressValue, RoomId, RoomLocation, Side,
};

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
    Room(RoomId, RoomLocation),
    ArenaItem(ItemLocation),
    DiscardPile(Side),
    /// A card has been scored and is currently resolving its scoring effects
    /// before moving to a score pile.
    Scoring,
    /// Card is in the [Side] player's score pile
    Scored(Side),
    /// A card has been played by the [Side] player and is in the process of
    /// resolving with the provided target
    Played(Side, CardTarget),
    /// A riftcaller card owned by a player in the game.
    Riftcaller(Side),
    /// Global modifier cards which change the rules of the game
    GameModifier,
}

impl CardPosition {
    /// Returns the [CardPositionKind] for this card
    pub fn kind(&self) -> CardPositionKind {
        self.into()
    }

    /// Returns true if this card is a riftcaller, is in a room, or has been
    /// played as an item.
    pub fn in_play(&self) -> bool {
        matches!(
            self.kind(),
            CardPositionKind::Riftcaller | CardPositionKind::Room | CardPositionKind::ArenaItem
        )
    }

    /// Returns true if this card is in a room
    pub fn in_room(&self) -> bool {
        self.kind() == CardPositionKind::Room
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

    /// True if this card is current in the indicated room
    pub fn is_room_occupant(&self, room_id: RoomId) -> bool {
        matches!(
            self,
            CardPosition::Room(room, location)
            if room_id == *room && *location == RoomLocation::Occupant
        )
    }

    /// Returns true if this card is in a user's score pile
    pub fn in_score_pile(&self) -> bool {
        self.kind() == CardPositionKind::Scored
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
    /// Is this card revealed to the [CardId.side] user?
    revealed_to_owner: bool,
    /// Is this card revealed to opponent of the [CardId.side] user?
    revealed_to_opponent: bool,
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
    pub data: CardData,
    /// Opaque value identifying this card's sort order within its CardPosition.
    /// Higher sorting keys are closer to the 'top' or 'front' of the position.
    pub sorting_key: u32,
    position: CardPosition,
}

impl CardState {
    /// Creates a new card state, placing the card into the `side` player's
    /// deck.
    pub fn new(id: CardId, variant: CardVariant) -> Self {
        Self {
            id,
            variant,
            position: CardPosition::DeckUnknown(id.side),
            sorting_key: 0,
            data: CardData {
                revealed_to_owner: false,
                revealed_to_opponent: false,
                is_face_up: false,
                ..CardData::default()
            },
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
                revealed_to_owner: is_face_up,
                revealed_to_opponent: is_face_up,
                is_face_up,
                ..CardData::default()
            },
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

    /// Clears all stored counters on this card.
    pub fn clear_all_counters(&mut self) {
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

    /// Returns true if this card is currently revealed to the indicated user
    ///
    /// Note that this is not the same as [Self::is_face_up], both players may
    /// know a card without it being the the 'face up' state.
    pub fn is_revealed_to(&self, side: Side) -> bool {
        if self.id.side == side {
            self.data.revealed_to_owner
        } else {
            self.data.revealed_to_opponent
        }
    }

    /// Change a card to the 'face up' state and makes the card revealed to both
    /// players.
    pub fn internal_turn_face_up(&mut self) {
        self.data.is_face_up = true;
        self.internal_set_revealed_to(Side::Overlord, true);
        self.internal_set_revealed_to(Side::Champion, true);
    }

    /// Change a card to the 'face down' state, but does *not* change its
    /// revealed state for either player.
    pub fn internal_turn_face_down(&mut self) {
        self.data.is_face_up = false;
    }

    /// Updates the 'revealed' state of a card to be visible to the indicated
    /// `side` player. Note that this is *not* the same as turning a card
    /// face-up, a card can be revealed to both players without being
    /// face-up
    pub fn internal_set_revealed_to(&mut self, side: Side, revealed: bool) {
        if self.id.side == side {
            self.data.revealed_to_owner = revealed
        } else {
            self.data.revealed_to_opponent = revealed
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
