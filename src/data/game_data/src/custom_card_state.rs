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

    /// Records an instance of this card being played.
    ///
    /// Cards store their [CardPlayId] while they are in play, but sometimes
    /// this information needs to persist longer. The most common example of
    /// this are spell cards, which go to a discard pile immediately but
    /// sometimes have ongoing effects on the game.
    RecordCardPlay { play_id: CardPlayId },

    /// Record that an enhancement cost has been paid for a given instance of
    /// playing this card. This is used to implement effects like "access the
    /// top 8 cards of the vault, you may pay <action> to access another."
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

    /// Returns the most recently recorded [CardPlayId] supplied via
    /// [CustomCardState::RecordCardPlay].
    pub fn last_recorded_card_play_id(&self) -> Option<CardPlayId> {
        self.list
            .iter()
            .rev()
            .filter_map(|state| match state {
                CustomCardState::RecordCardPlay { play_id } => Some(*play_id),
                _ => None,
            })
            .next()
    }

    /// Returns true if a [CustomCardState::PaidForEnhancement] entry has been
    /// recorded for this [CardPlayId].
    pub fn paid_for_enhancement(&self, id: CardPlayId) -> bool {
        self.list.iter().rev().any(
            |state| matches!(state,
                CustomCardState::PaidForEnhancement { play_id } if id == *play_id),
        )
    }
}
