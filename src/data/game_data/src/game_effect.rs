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

use core_data::game_primitives::{
    AbilityId, ActionCount, CardId, CurseCount, DamageAmount, GameObjectId, InitiatedBy, ManaValue,
    PowerChargeValue, RoomId, Side,
};
use serde::{Deserialize, Serialize};

use crate::card_state::CardPosition;
use crate::custom_card_state::CustomCardState;
use crate::game_actions::CardTarget;
use crate::prompt_data::FromZone;

/// An arbitrary modification to the state of an ongoing game.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum GameEffect {
    /// Proceed without taking any action
    Continue,
    /// Stop the current 'play card' game action
    AbortPlayingCard,
    /// Requests to play the `CardConfig::choice_effect` visual effect for
    /// `owner` on the indicated `target`.
    PlayChoiceEffect { owner: CardId, target: GameObjectId },
    /// Have the [Side] player draw some number of cards
    DrawCards(Side, u32, InitiatedBy),
    /// Sacrifice the indicated permanent, moving it to its owner's discard
    /// pile.
    SacrificeCard(CardId),
    /// Destroy the indicated permanent
    DestroyCard(CardId, InitiatedBy),
    /// A mana cost for a prompt choice. Choices will not be shown if the player
    /// is unable to pay their costs.
    ManaCost(Side, ManaValue, InitiatedBy),
    /// An action point cost for a prompt choice. Choices will not be shown if
    /// the player is unable to pay their costs.
    ActionCost(Side, ActionCount),
    /// Deal damage to the Riftcaller as a cost. Choice will not be shown if it
    /// would cause this player to lose the game.
    TakeDamageCost(AbilityId, u32),
    /// Initiate a new raid on this room.
    InitiateRaid(RoomId, AbilityId),
    /// End the current raid in failure.
    EndRaid(AbilityId),
    /// End the current custom access event
    EndCustomAccess(AbilityId),
    /// Move a card to a new target position
    MoveCard(CardId, CardPosition),
    /// Prevent *up to* this amount of incoming damage if there is an active
    /// damage event.
    PreventDamage(DamageAmount),
    /// Prevent *up to* this quantity of incoming curses if there is an active
    /// curse event.
    PreventCurses(CurseCount),
    /// Prevent the [CardId] card from being destroyed if it is currently the
    /// target of a destruction event.
    PreventDestroyingCard(CardId),
    /// Adds a card to the 'side' player's 'prompt_selected_cards' list.
    SelectCardForPrompt(Side, CardId),
    /// Removes all selected cards for the 'side' player.
    ClearAllSelectedCards(Side),
    /// Adds a new prompt request for the [Side] player for [AbilityId]. The
    /// prompt will be supplied with the provided index via
    /// `PromptData::Index`.
    PushPromptWithIndex(Side, AbilityId, u32),
    /// Swap a card's position with the last card in the 'side' player's
    /// 'prompt_selected_cards' list. Returns an error if no card is selected.
    /// Removes the chosen card from the 'prompt_selected_cards' list.
    SwapWithSelected(Side, CardId),
    /// Appends a new [CustomCardState] entry for this card.
    AppendCustomCardState(CardId, CustomCardState),
    /// Evade the current raid encounter, jumping to the next raid state
    EvadeCurrentEncounter,
    /// Put a card into play for no mana cost
    PlayCardForNoMana(CardId, CardTarget, FromZone, InitiatedBy),
    /// Prevent the current raid, if any, from accessing cards
    PreventRaidCardAccess,
    /// Change the current raid target to the indicated room.
    ChangeRaidTarget(RoomId, InitiatedBy),
    /// Defeat the minion currently being encountered during a raid.
    DefeatCurrentMinion,
    /// Reveal the indicated card. This is an explicit game event which must use
    /// the word "reveal" in card text.
    RevealCard(CardId),
    /// Add power charge counters to a card
    AddPowerCharges(CardId, PowerChargeValue),
}

impl GameEffect {
    pub fn is_secondary(&self) -> bool {
        match self {
            Self::Continue | Self::AbortPlayingCard | Self::EndCustomAccess(..) => true,
            _ => false,
        }
    }
}
