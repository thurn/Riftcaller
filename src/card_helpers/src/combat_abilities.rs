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

use core_data::game_primitives::{ActionCount, DamageAmount, InitiatedBy, ManaValue, Side};
use game_data::card_definition::{Ability, AbilityType};
use game_data::delegate_data::RaidOutcome;
use game_data::text::TextToken::*;
use rules::mana::ManaPurpose;
use rules::{deal_damage, mana, mutations};

use crate::text_macro::text;
use crate::{text_helpers, this};

/// Minion combat ability which deals damage to the Champion player during
/// combat, causing them to discard `N` random cards and lose the game if they
/// cannot.
pub fn combat_deal_damage<const N: DamageAmount>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text_helpers::named_trigger(Combat, text![DealDamage(N)]),
        delegates: vec![this::combat(|g, s, _| deal_damage::apply(g, s, N))],
    }
}

/// Minion combat ability which causes the Champion to lose `N` mana.
pub fn combat_lose_mana<const N: ManaValue>() -> Ability {
    Ability::new_with_delegate(
        text![text_helpers::named_trigger(Combat, text!["The Champion", LosesMana(N)])],
        this::combat(|g, s, _| {
            mana::lose_upto(g, Side::Champion, s.initiated_by(), ManaPurpose::CombatAbility, N)
        }),
    )
}

/// Minion combat ability which ends the current raid in failure.
pub fn combat_end_raid() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text_helpers::named_trigger(Combat, text!["End the raid"]),
        delegates: vec![this::combat(|g, s, _| {
            mutations::end_raid(g, InitiatedBy::Ability(s.ability_id()), RaidOutcome::Failure)
        })],
    }
}

/// Minion combat ability which gains mana
pub fn combat_gain_mana<const N: ManaValue>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text_helpers::named_trigger(Combat, text![GainMana(N)]),
        delegates: vec![this::combat(|g, _, _| {
            mana::gain(g, Side::Overlord, N);
            Ok(())
        })],
    }
}

/// Minion combat ability which causes the Champion player to lose action
/// points.
pub fn combat_lose_action_points<const N: ActionCount>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text_helpers::named_trigger(Combat, text!["Remove", Actions(1)]),
        delegates: vec![this::combat(|g, _s, _| {
            mutations::lose_action_points_if_able(g, Side::Champion, N)
        })],
    }
}