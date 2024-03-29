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

use card_definition_data::ability_data::{Ability, AbilityType};
use core_data::game_primitives::{ActionCount, DamageAmount, InitiatedBy, ManaValue, Side};
use game_data::delegate_data::RaidOutcome;
use game_data::game_actions::ButtonPromptContext;
use game_data::game_effect::GameEffect;
use game_data::prompt_data::PromptChoice;
use game_data::text::TextToken::*;
use rules::mana::ManaPurpose;
use rules::{damage, end_raid, mana, mutations, prompts};

use crate::text_macro::text;
use crate::{abilities, show_prompt, text_helpers, this};

/// Minion combat ability which deals damage to the Riftcaller player during
/// combat, causing them to discard `N` random cards and lose the game if they
/// cannot.
pub fn deal_damage<const N: DamageAmount>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text_helpers::named_trigger(Combat, text![DealDamage(N)]),
        delegates: abilities::game(vec![this::combat(|g, s, _| damage::deal(g, s, N))]),
    }
}

/// Minion combat ability which causes the Riftcaller to lose `N` mana.
pub fn lose_mana<const N: ManaValue>() -> Ability {
    Ability::new_with_delegate(
        text![text_helpers::named_trigger(Combat, text!["The Riftcaller", LosesMana(N)])],
        this::combat(|g, s, _| {
            mana::lose_upto(g, Side::Riftcaller, s.initiated_by(), ManaPurpose::CombatAbility, N)
        }),
    )
}

/// Minion combat ability which ends the current raid in failure.
pub fn end_raid() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text_helpers::named_trigger(Combat, text!["End the raid"]),
        delegates: abilities::game(vec![this::combat(|g, s, _| {
            end_raid::run(g, InitiatedBy::Ability(s.ability_id()), RaidOutcome::Failure)
        })]),
    }
}

/// Minion combat ability which gains mana
pub fn gain_mana<const N: ManaValue>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text_helpers::named_trigger(Combat, text![GainMana(N)]),
        delegates: abilities::game(vec![this::combat(|g, _, _| {
            mana::gain(g, Side::Covenant, N);
            Ok(())
        })]),
    }
}

/// Minion combat ability which causes the Riftcaller player to lose action
/// points.
pub fn lose_action_points<const N: ActionCount>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text_helpers::named_trigger(Combat, text!["Remove", Actions(1)]),
        delegates: abilities::game(vec![this::combat(|g, _, _| {
            mutations::lose_action_points_if_able(g, Side::Riftcaller, N)
        })]),
    }
}

/// Minion combat ability which allows an artifact to be destroyed
pub fn destroy_artifact() -> Ability {
    Ability::new(text_helpers::named_trigger(Combat, text!["Destroy an artifact"]))
        .delegate(this::combat(|g, s, _| {
            prompts::push(g, Side::Covenant, s);
            Ok(())
        }))
        .delegate(this::prompt(|g, s, _, _| {
            show_prompt::with_context_and_choices(
                ButtonPromptContext::Card(s.card_id()),
                g.artifacts()
                    .map(|card| {
                        PromptChoice::new()
                            .effect(GameEffect::DestroyCard(card.id, s.initiated_by()))
                            .anchor_card(card.id)
                    })
                    .collect(),
            )
        }))
}
