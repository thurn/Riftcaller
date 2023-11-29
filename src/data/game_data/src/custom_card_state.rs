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

use core_data::game_primitives::{CardId, CardPlayId};
use serde::{Deserialize, Serialize};

use crate::game_state::TurnData;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CustomCardState {
    /// Affect some card for the duration of the provided [CardPlayId] being
    /// valid.
    ///
    /// The source of provided ID is unspecified. It may for example be from the
    /// parent card for "while this card is in play, do X" effects, but it could
    /// also be from the target card for "while this target is in play, apply
    /// X".
    TargetCard { target_card: CardId, play_id: CardPlayId },

    /// Apply an effect to a card for the duration of a single turn.
    TargetCardForTurn { target_card: CardId, turn: TurnData },

    /// Record that an enhancement cost has been paid for a given instance of
    /// playing this card. This is used to implement effects like "access the
    /// top 8 cards of the vault, you may pay an action to access another."
    PaidForEnhancement { play_id: CardPlayId },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomCardStateList {
    list: Vec<CustomCardState>,
}

impl CustomCardStateList {
    pub fn push(&mut self, state: CustomCardState) {
        self.list.push(state);
    }

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

    /// Returns true if a [CustomCardState::PaidForEnhancement] entry has been
    /// recorded for this [CardPlayId].
    pub fn paid_for_enhancement(&self, id: CardPlayId) -> bool {
        self.list.iter().rev().any(|state| {
            matches!(state,
                CustomCardState::PaidForEnhancement { play_id } if id == *play_id)
        })
    }
}
