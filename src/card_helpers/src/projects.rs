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

use game_data::card_definition::{Ability, AbilityType, CardConfig};
use game_data::delegates::{Delegate, EventDelegate, QueryDelegate};
use game_data::primitives::CardSubtype;
use rules::{mutations, queries};

use crate::text;

/// Marks a project as an 'activated project', which must have it unveil cost
/// paid before it can be turned face up.
pub fn activated() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text![],
        delegates: vec![
            activate_while_face_down(),
            face_down_ability_cost(),
            unveil_when_activated(),
        ],
    }
}

pub fn activated_config() -> CardConfig {
    CardConfig { subtypes: vec![CardSubtype::Activated], ..CardConfig::default() }
}

pub fn triggered_config() -> CardConfig {
    CardConfig { subtypes: vec![CardSubtype::Triggered], ..CardConfig::default() }
}

/// Marks a card's abilities as possible to activate while it is face-down
pub fn activate_while_face_down() -> Delegate {
    Delegate::CanActivateWhileFaceDown(QueryDelegate {
        requirement: crate::this_card,
        transformation: |_g, _, _, current| current.with_override(true),
    })
}

/// Makes an ability's mana cost equal to the cost of its parent card while that
/// card is face-down.
pub fn face_down_ability_cost() -> Delegate {
    Delegate::AbilityManaCost(QueryDelegate {
        requirement: crate::this_card,
        transformation: |g, s, _, current| {
            if g.card(s.card_id()).is_face_up() {
                current
            } else {
                Some(current.unwrap_or(0) + queries::mana_cost(g, s.card_id())?)
            }
        },
    })
}

/// Turns a card face up without paying its mana cost when any ability is
/// activated. Usually combined with [face_down_ability_cost] to incorporate the
/// unveil cost into the activation cost.
pub fn unveil_when_activated() -> Delegate {
    Delegate::ActivateAbility(EventDelegate {
        requirement: crate::this_card,
        mutation: |g, s, _| {
            mutations::unveil_project_ignoring_costs(g, s.card_id())?;
            Ok(())
        },
    })
}
