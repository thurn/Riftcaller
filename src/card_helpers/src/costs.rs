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

use game_data::card_definition::{Cost, CustomCost};
use game_data::card_state::CardCounter;
use game_data::primitives::{AbilityId, ActionCount, CardId, ManaValue};
use game_data::text::{TextElement, TextToken};
use rules::mutations;

use crate::history;

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

/// Cost for an ability which costs 1 action point and requires the owning card
/// to be sacrificed.
pub fn sacrifice_for_action() -> Cost<AbilityId> {
    Cost { mana: None, actions: 1, custom_cost: sacrifice_cost() }
}

/// Cost for an ability which requires the owning card to be sacrificed.
pub fn sacrifice() -> Cost<AbilityId> {
    Cost { mana: None, actions: 0, custom_cost: sacrifice_cost() }
}

/// Cost for an ability which costs power charges to use.
pub fn power_charges<const N: u32>() -> Cost<AbilityId> {
    Cost {
        mana: None,
        actions: 0,
        custom_cost: Some(CustomCost {
            can_pay: |g, id| g.card(id.card_id).counters(CardCounter::PowerCharges) >= N,
            pay: |g, id| mutations::spend_power_charges(g, id.card_id, N),
            description: Some(TextElement::Token(TextToken::PowerCharges(N))),
        }),
    }
}

/// A [CustomCost] which allows an ability to be activated by sacrificing the
/// card.
pub fn sacrifice_cost() -> Option<CustomCost<AbilityId>> {
    Some(CustomCost {
        can_pay: |game, ability_id| {
            game.card(ability_id.card_id).is_face_up()
                && game.card(ability_id.card_id).position().in_play()
        },
        pay: |game, ability_id| mutations::sacrifice_card(game, ability_id.card_id),
        description: Some(TextElement::Token(TextToken::SacrificeCost)),
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

/// [Cost] for a scheme card
pub fn scheme() -> Cost<CardId> {
    Cost { mana: None, actions: 1, custom_cost: None }
}
