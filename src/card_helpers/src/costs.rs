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

use core_data::game_primitives::{AbilityId, ActionCount, CardId, ManaValue};
use game_data::card_configuration::{Cost, CustomCost};
use game_data::card_state::CardCounter;
use game_data::text::{TextElement, TextToken};
use rules::mutations;

use crate::{history, text};

/// Provides the cost for a card, with 1 action point required and `mana` mana
/// points
pub fn mana(mana: ManaValue) -> Cost<CardId> {
    Cost { mana: Some(mana), actions: 1, custom_cost: None }
}

/// Provides the cost for a an ability, with no action points required and
/// `mana` mana cost.
pub fn ability_mana(mana: ManaValue) -> Cost<AbilityId> {
    Cost { mana: Some(mana), actions: 0, custom_cost: None }
}

/// [Cost] for a scheme card
pub fn scheme() -> Cost<CardId> {
    Cost { mana: None, actions: 1, custom_cost: None }
}

/// [Cost] for a riftcaller or chapter card
pub fn identity() -> Cost<CardId> {
    Cost { mana: None, actions: 0, custom_cost: None }
}

/// Cost for an ability which costs 1 action point and requires the owning card
/// to be sacrificed.
pub fn sacrifice_and_action() -> Cost<AbilityId> {
    Cost { mana: None, actions: 1, custom_cost: sacrifice_custom_cost() }
}

/// Cost for an ability which requires the owning card to be sacrificed.
pub fn sacrifice() -> Cost<AbilityId> {
    Cost { mana: None, actions: 0, custom_cost: sacrifice_custom_cost() }
}

/// A [CustomCost] which allows an ability to be activated by sacrificing the
/// card.
pub fn sacrifice_custom_cost() -> Option<CustomCost<AbilityId>> {
    Some(CustomCost {
        can_pay: |game, ability_id| {
            game.card(ability_id.card_id).is_face_up()
                && game.card(ability_id.card_id).position().in_play()
        },
        pay: |game, ability_id| mutations::sacrifice_card(game, ability_id.card_id),
        description: Some(TextElement::Token(TextToken::SacrificeCost)),
    })
}

/// Cost for an ability which requires the owning card to be banished.
pub fn banish() -> Cost<AbilityId> {
    Cost { mana: None, actions: 0, custom_cost: banish_custom_cost() }
}

/// A [CustomCost] which allows an ability to be activated by banishing the
/// card.
pub fn banish_custom_cost() -> Option<CustomCost<AbilityId>> {
    Some(CustomCost {
        can_pay: |game, ability_id| {
            game.card(ability_id.card_id).is_face_up()
                && game.card(ability_id.card_id).position().in_play()
        },
        pay: |game, ability_id| mutations::banish_card(game, ability_id.card_id),
        description: Some(TextElement::Children(text![TextToken::Banish, "this card"])),
    })
}

/// Cost for an ability which costs power charges to use.
pub fn power_charges<const N: u32>() -> Cost<AbilityId> {
    Cost { mana: None, actions: 0, custom_cost: power_charges_custom_cost::<N>() }
}

pub fn power_charges_and_action<const N: u32>() -> Cost<AbilityId> {
    Cost { mana: None, actions: 1, custom_cost: power_charges_custom_cost::<N>() }
}

/// A [CustomCost] for an ability which costs power charges to use.
pub fn power_charges_custom_cost<const N: u32>() -> Option<CustomCost<AbilityId>> {
    Some(CustomCost {
        can_pay: |g, id| g.card(id.card_id).counters(CardCounter::PowerCharges) >= N,
        pay: |g, id| mutations::spend_power_charges(g, id.card_id, N),
        description: Some(TextElement::Token(TextToken::PowerCharges(N))),
    })
}

/// Cost for an ability which costs progress counters to use.
pub fn progress_counters<const N: u32>() -> Cost<AbilityId> {
    Cost { mana: None, actions: 0, custom_cost: progress_counters_custom_cost::<N>() }
}

/// A [CustomCost] for an ability which costs progress counters to use.
pub fn progress_counters_custom_cost<const N: u32>() -> Option<CustomCost<AbilityId>> {
    Some(CustomCost {
        can_pay: |g, id| g.card(id.card_id).counters(CardCounter::Progress) >= N,
        pay: |g, id| g.card_mut(id.card_id).remove_counters_or_error(CardCounter::Progress, N),
        description: Some(TextElement::Token(TextToken::ProgressCounters(N))),
    })
}

/// A [CustomCost] which allows an ability to be activated once per turn.
pub fn once_per_turn() -> Option<CustomCost<AbilityId>> {
    Some(CustomCost {
        can_pay: |g, ability_id| {
            history::ability_activations_this_turn(g, ability_id).next().is_none()
        },
        pay: |_, _| Ok(()),
        description: None,
    })
}

/// A [Cost] which requires no mana and `actions` action points.
pub fn actions(actions: ActionCount) -> Cost<AbilityId> {
    Cost { mana: None, actions, custom_cost: None }
}
