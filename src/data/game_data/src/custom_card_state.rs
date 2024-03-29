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

use core_data::game_primitives::{CardId, CardPlayId, CardType, MinionEncounterId, RoomId};
use serde::{Deserialize, Serialize};

use crate::game_state::TurnData;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CustomCardState {
    /// Affect some card based on the scope of a given [CardPlayId].
    ///
    /// The source of provided [CardPlayId] is unspecified -- it may be
    /// associated with either the parent card or the target card, for example.
    TargetCard { target_card: CardId, play_id: CardPlayId },

    /// Apply an effect to a card for the duration of a single turn.
    TargetCardForTurn { target_card: CardId, turn: TurnData },

    /// Affect some room based on the scope of a given [CardPlayId].
    TargetRoom { target_room: RoomId, play_id: CardPlayId },

    /// Record that an enhancement cost has been paid for a given instance of
    /// playing this card. This is used to implement effects like "access the
    /// top 8 cards of the vault, you may pay an action to access another."
    PaidForEnhancement { play_id: CardPlayId },

    /// A card or ability's effect should be applied for the duration of the
    /// minion encounter with the provided [MinionEncounterId].
    ActiveForEncounter { encounter_id: MinionEncounterId },

    /// A Riftcaller's ability has triggered in the indicated turn.
    IdentityTriggeredForTurn { turn: TurnData },

    /// An ability which triggers once per turn while a card is in play has triggered
    InPlayAbilityTriggeredForTurn { turn: TurnData, play_id: CardPlayId },

    /// A card type selected for the duration of a given turn
    CardTypeForTurn { card_type: CardType, turn: TurnData },
}

/// Records custom state entries for a given card.
///
/// This keeps track of miscellaneous state related to resolving a card's
/// abilities, such as targets which have been selected for this card. It is
/// designed as an "append-only" data structure, meaning that state entries are
/// never removed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomCardStateList {
    list: Vec<CustomCardState>,
}

impl CustomCardStateList {
    /// Add a new [CustomCardState] entry.
    pub fn push(&mut self, state: CustomCardState) {
        self.list.push(state);
    }

    /// Mark all of the provided cards as targets for the given [CardPlayId] via
    /// [CustomCardState::TargetCard].
    pub fn record_targets(&mut self, play_id: CardPlayId, targets: &[CardId]) {
        for target_card in targets {
            self.push(CustomCardState::TargetCard { target_card: *target_card, play_id })
        }
    }

    /// Returns an iterator over all [CustomCardState::TargetCard] targets which
    /// have been recorded for a given [CardPlayId].
    pub fn targets(&self, id: CardPlayId) -> impl Iterator<Item = CardId> + '_ {
        self.list.iter().filter_map(move |state| match state {
            CustomCardState::TargetCard { target_card, play_id } if *play_id == id => {
                Some(*target_card)
            }
            _ => None,
        })
    }

    /// Returns an iterator over [CustomCardState::TargetCardForTurn] targets
    /// which have been recorded for a given [TurnData].
    pub fn targets_for_turn(&self, t: TurnData) -> impl Iterator<Item = CardId> + '_ {
        self.list.iter().filter_map(move |state| match state {
            CustomCardState::TargetCardForTurn { target_card, turn } if *turn == t => {
                Some(*target_card)
            }
            _ => None,
        })
    }

    /// Returns an iterator over [CustomCardState::TargetCardForTurn] targets
    /// which have been recorded for a given [TurnData] or the turn immediately
    /// prior to it.
    pub fn targets_for_turn_cycle(&self, t: TurnData) -> impl Iterator<Item = CardId> + '_ {
        let previous = t.previous();
        self.list.iter().filter_map(move |state| match state {
            CustomCardState::TargetCardForTurn { target_card, turn }
                if *turn == t || Some(*turn) == previous =>
            {
                Some(*target_card)
            }
            _ => None,
        })
    }

    /// Checks if the provided [CardId] is registered as a target for the given
    /// [CardPlayId].
    ///
    /// Returns false if the [CardPlayId] is `None`, e.g. because the parent
    /// card is no longer in play.
    pub fn targets_contain(&self, id: Option<CardPlayId>, card_id: CardId) -> bool {
        if let Some(play_id) = id {
            self.targets(play_id).any(|id| id == card_id)
        } else {
            false
        }
    }

    /// Returns an iterator over all [CustomCardState::TargetRoom] targets which
    /// have been recorded for a given [CardPlayId].
    pub fn target_rooms(&self, id: CardPlayId) -> impl Iterator<Item = RoomId> + '_ {
        self.list.iter().filter_map(move |state| match state {
            CustomCardState::TargetRoom { target_room, play_id } if *play_id == id => {
                Some(*target_room)
            }
            _ => None,
        })
    }

    /// Checks if the provided [RoomId] is registered as a room target for the
    /// given [CardPlayId].
    ///
    /// Returns false if the [CardPlayId] is `None`.
    pub fn target_rooms_contain(&self, id: Option<CardPlayId>, room_id: RoomId) -> bool {
        if let Some(play_id) = id {
            self.target_rooms(play_id).any(|id| id == room_id)
        } else {
            false
        }
    }

    /// Returns true if a [CustomCardState::PaidForEnhancement] entry has been
    /// recorded for this [CardPlayId].
    pub fn paid_for_enhancement(&self, id: CardPlayId) -> bool {
        self.list.iter().rev().any(|state| {
            matches!(state,
                CustomCardState::PaidForEnhancement { play_id } if id == *play_id)
        })
    }

    /// Returns true if a [CustomCardState::ActiveForEncounter] entry has been
    /// recorded for this [MinionEncounterId].
    pub fn is_active_for_encounter(&self, id: MinionEncounterId) -> bool {
        self.list.iter().rev().any(|state| {
            matches!(state,
                CustomCardState::ActiveForEncounter { encounter_id } if id == *encounter_id)
        })
    }

    /// Returns true if a [CustomCardState::InPlayAbilityTriggeredForTurn] entry
    /// has been recorded for the provided turn and card play id.
    pub fn in_play_ability_triggered_for_turn(
        &self,
        turn_data: TurnData,
        card_play_id: CardPlayId,
    ) -> bool {
        self.list.iter().rev().any(|state| {
            matches!(state,
                CustomCardState::InPlayAbilityTriggeredForTurn { turn, play_id }
                if turn_data == *turn && card_play_id == *play_id)
        })
    }

    /// Returns true if a [CustomCardState::IdentityTriggeredForTurn] entry
    /// has been recorded for the provided turn.
    pub fn identity_triggered_for_turn(&self, turn_data: TurnData) -> bool {
        self.list.iter().rev().any(|state| {
            matches!(state,
                CustomCardState::IdentityTriggeredForTurn { turn } if turn_data == *turn)
        })
    }

    /// Returns the chosen card type if a [CustomCardState::CardTypeForTurn]
    /// entry has been recorded for the provided turn.
    pub fn card_type_for_turn(&self, turn_data: TurnData) -> Option<CardType> {
        self.list
            .iter()
            .rev()
            .filter_map(|state| match state {
                CustomCardState::CardTypeForTurn { card_type, turn } if *turn == turn_data => {
                    Some(*card_type)
                }
                _ => None,
            })
            .next()
    }
}
