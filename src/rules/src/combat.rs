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

//! Calculations for using weapons during combat

use core_data::game_primitives::{AttackValue, CardId, ManaValue};
use game_data::card_definition::{AttackBoost, CustomBoostCost, CustomWeaponCost};
use game_data::card_state::CardCounter;
use game_data::delegate_data::{
    AttackBoostBonusQuery, CanDefeatTargetQuery, CanEncounterTargetQuery, CardEncounter,
};
use game_data::flag_data::Flag;
use game_data::game_state::GameState;

use crate::mana::ManaPurpose;
use crate::{dispatch, mana, queries};

/// Records the number of times some [CustomBoostCost] must be paid to defeat a
/// minion.
pub struct CustomBoostActivation {
    pub activation_count: u32,
    pub cost: CustomBoostCost,
}

/// Result of a call to [cost_to_defeat_target].
pub struct CostToDefeatTarget {
    /// Mana required to defeat the minion
    pub mana_cost: ManaValue,
    /// Attack value added to this weapon to defeat this minion
    pub attack_boost: AttackValue,
    /// Additional cost to use this weapon
    pub custom_weapon_cost: Option<CustomWeaponCost>,
    /// Custom boost costs required to defeat this minion, if any.
    pub custom_boost_activation: Option<CustomBoostActivation>,
}

/// Returns the costs the owner of `card_id` would need to spend to raise its
/// [AttackValue] to the provided `target` by activating its weapon boost
/// ability. See [CostToDefeatTarget].
///
/// - Returns a mana cost of 0 if this card can already defeat the target.
/// - Returns None if it is impossible for this card to defeat the target.
pub fn cost_to_defeat_target(
    game: &GameState,
    card_id: CardId,
    target_id: CardId,
) -> Option<CostToDefeatTarget> {
    let target = queries::health(game, target_id);
    let current = queries::base_attack(game, card_id);

    // Handle custom weapon costs
    let custom_weapon_cost = if let Some(custom) =
        queries::attack_boost(game, card_id).and_then(|b| b.custom_weapon_cost.as_ref())
    {
        if !can_pay_custom_weapon_cost(game, card_id, custom) {
            return None;
        }
        Some(custom.clone())
    } else {
        None
    };

    let mut result = if current >= target {
        CostToDefeatTarget {
            mana_cost: 0,
            attack_boost: 0,
            custom_weapon_cost,
            custom_boost_activation: None,
        }
    } else {
        let Some(boost) = queries::attack_boost(game, card_id) else {
            return None;
        };

        let bonus = attack_boost_bonus(game, card_id, boost);
        if bonus == 0 {
            return None;
        } else {
            let increase = target - current;
            // If the boost does not evenly divide into the target, we need to apply it an
            // additional time.
            let add = if (increase % bonus) == 0 { 0 } else { 1 };

            #[allow(clippy::integer_division)] // Deliberate integer truncation
            let boost_count = add + (increase / bonus);

            // Handle applying custom (non-mana) weapon boost abilities
            let custom_boost_activation = if let Some(custom) = &boost.custom_boost_cost {
                if !can_pay_custom_boost_cost(game, card_id, custom, boost_count) {
                    return None;
                }
                Some(CustomBoostActivation { activation_count: boost_count, cost: custom.clone() })
            } else {
                None
            };

            CostToDefeatTarget {
                mana_cost: boost_count * boost.cost,
                attack_boost: boost_count * bonus,
                custom_weapon_cost,
                custom_boost_activation,
            }
        }
    };

    result.mana_cost += queries::shield(game, target_id, Some(card_id))
        .saturating_sub(queries::breach(game, card_id));
    Some(result)
}

/// Can the `source` card defeat the `target` card in an encounter by paying its
/// shield cost and dealing enough damage to equal its health (potentially after
/// paying mana & applying boosts), or via some other game mechanism?
pub fn can_defeat_target(game: &GameState, source: CardId, target: CardId) -> bool {
    if !can_encounter_target(game, source, target) {
        return false;
    }
    let Some(cost_to_defeat) = cost_to_defeat_target(game, source, target) else {
        return false;
    };

    let can_defeat = cost_to_defeat.mana_cost
        <= mana::get(game, source.side, ManaPurpose::UseWeapon(source))
        && cost_to_defeat.custom_boost_activation.as_ref().map_or(true, |custom| {
            can_pay_custom_boost_cost(game, source, &custom.cost, custom.activation_count)
        });

    dispatch::perform_query(
        game,
        CanDefeatTargetQuery(CardEncounter::new(source, target)),
        Flag::new(can_defeat),
    )
    .into()
}

/// Whether the provided `source` card is able to target the `target` card with
/// an encounter action. Typically used to determine whether a weapon can target
/// a minion, e.g. based on resonance.
pub fn can_encounter_target(game: &GameState, weapon: CardId, minion: CardId) -> bool {
    let Some(weapon_resonance) = queries::resonance(game, weapon) else {
        return false;
    };
    let Some(minion_resonance) = queries::resonance(game, minion) else {
        return false;
    };

    let can_encounter = weapon_resonance.prismatic
        || (weapon_resonance.mortal && minion_resonance.mortal)
        || (weapon_resonance.infernal && minion_resonance.infernal)
        || (weapon_resonance.astral && minion_resonance.astral);

    dispatch::perform_query(
        game,
        CanEncounterTargetQuery(CardEncounter::new(weapon, minion)),
        Flag::new(can_encounter),
    )
    .into()
}

/// Queries the amount of attack to add to a card each time its weapon boost
/// ability is activated.
pub fn attack_boost_bonus(game: &GameState, card_id: CardId, boost: &AttackBoost) -> AttackValue {
    dispatch::perform_query(game, AttackBoostBonusQuery(card_id), boost.bonus)
}

fn can_pay_custom_weapon_cost(game: &GameState, card_id: CardId, cost: &CustomWeaponCost) -> bool {
    match cost {
        CustomWeaponCost::ActionPoints(points) => game.player(card_id.side).actions >= *points,
    }
}

fn can_pay_custom_boost_cost(
    game: &GameState,
    card_id: CardId,
    cost: &CustomBoostCost,
    times: u32,
) -> bool {
    match cost {
        CustomBoostCost::PowerCharges(n) => {
            game.card(card_id).counters(CardCounter::PowerCharges) >= (times * n)
        }
    }
}
