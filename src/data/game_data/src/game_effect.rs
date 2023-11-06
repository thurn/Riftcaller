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

use serde::{Deserialize, Serialize};

use crate::card_state::CardPosition;
use crate::primitives::{
    AbilityId, ActionCount, CardId, CurseCount, DamageAmount, ManaValue, RoomId, Side,
};

/// An arbitrary modification to the state of an ongoing game.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum GameEffect {
    /// Proceed without taking any action
    Continue,
    /// Stop the current 'play card' game action
    AbortPlayingCard,
    /// Sacrifice the indicated permanent, moving it to its owner's discard
    /// pile.
    SacrificeCard(CardId),
    /// Destroy the indicated permanent
    DestroyCard(CardId),
    /// A player loses mana
    LoseMana(Side, ManaValue),
    /// A player loses action points
    LoseActions(Side, ActionCount),
    /// Initiate a new raid on this room.
    InitiateRaid(RoomId, AbilityId),
    /// End the current raid in failure.
    EndRaid,
    /// Deal damage to the Champion
    TakeDamage(AbilityId, u32),
    /// Move a card to a new target position
    MoveCard(CardId, CardPosition),
    /// Prevent *up to* this amount of incoming damage if there is an active
    /// damage event.
    PreventDamage(DamageAmount),
    /// Prevent *up to* this quantity of incoming curses if there is an active
    /// curse event.
    PreventCurses(CurseCount),
    /// Adds a card to the 'side' player's 'prompt_selected_cards' list.
    SelectCardForPrompt(Side, CardId),
    /// Swap a card's position with the last card in the 'side' player's
    /// 'prompt_selected_cards' list. Returns an error if no card is selected.
    /// Removes the chosen card from the 'prompt_selected_cards' list.
    SwapWithSelected(Side, CardId),
}

impl GameEffect {
    pub fn is_secondary(&self) -> bool {
        match self {
            Self::Continue | Self::AbortPlayingCard => true,
            _ => false,
        }
    }
}
