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

//! Helpers for defining common card abilities

use game_data::card_definition::{Ability, AbilityType, Cost, TargetRequirement};
use game_data::card_name::CardMetadata;
use game_data::card_state::CardPosition;
use game_data::delegates::{Delegate, EventDelegate, QueryDelegate, RaidOutcome, Scope};
use game_data::game::GameState;
use game_data::primitives::{AbilityId, AttackValue, CardId, DamageAmount, ManaValue};
use game_data::text::TextToken::*;
use rules::mutations::OnZeroStored;
use rules::{mutations, queries};

use crate::text_macro::text;
use crate::*;

/// Helper to flatten a list of `Option` and remove `None` values.
pub fn some(abilities: Vec<Option<Ability>>) -> Vec<Ability> {
    abilities.into_iter().flatten().collect()
}

/// Creates a standard [Ability] with a single [Delegate].
pub fn standard(text: Vec<TextElement>, delegate: Delegate) -> Ability {
    Ability { text, ability_type: AbilityType::Standard, delegates: vec![delegate] }
}

/// Returns the provided [Ability] only for upgraded versions of a card.
pub fn when_upgraded(metadata: CardMetadata, ability: Ability) -> Option<Ability> {
    metadata.upgraded.then_some(ability)
}

pub fn silent_ability(ability: Ability) -> Ability {
    Ability { text: text![], ..ability }
}

/// The standard weapon ability; applies an attack boost for the duration of a
/// single encounter.
pub fn encounter_boost() -> Ability {
    Ability {
        ability_type: AbilityType::Encounter,
        text: encounter_ability_text(text![EncounterBoostCost], text![EncounterBoostBonus]),
        delegates: vec![
            Delegate::ActivateBoost(EventDelegate::new(this_card, mutations::write_boost)),
            Delegate::AttackValue(QueryDelegate::new(this_card, add_boost)),
            Delegate::EncounterEnd(EventDelegate::new(always, mutations::clear_boost)),
        ],
    }
}

/// Applies this card's `attack_boost` stat a number of times equal to its
/// [CardState::boost_count]. Returns default if this card has no attack boost
/// defined.
fn add_boost(game: &GameState, _: Scope, card_id: &CardId, current: AttackValue) -> AttackValue {
    let boost_count = queries::boost_count(game, *card_id);
    let bonus = queries::attack_boost(game, *card_id).unwrap_or_default().bonus;
    current + (boost_count * bonus)
}

/// Store `N` mana in this card when played. Move it to the discard pile when
/// the stored mana is depleted.
pub fn store_mana_on_play<const N: ManaValue>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: trigger_text(Play, text![StoreMana(N)]),
        delegates: vec![
            Delegate::PlayCard(EventDelegate::new(this_card, |g, _s, played| {
                g.card_mut(played.card_id).data.stored_mana = N;
                Ok(())
            })),
            Delegate::StoredManaTaken(EventDelegate::new(this_card, |g, s, card_id| {
                if g.card(*card_id).data.stored_mana == 0 {
                    mutations::move_card(g, *card_id, CardPosition::DiscardPile(s.side()))
                } else {
                    Ok(())
                }
            })),
        ],
    }
}

/// Activated ability to take `N` stored mana from this card by paying a cost
pub fn activated_take_mana<const N: ManaValue>(cost: Cost<AbilityId>) -> Ability {
    Ability {
        ability_type: AbilityType::Activated(cost, TargetRequirement::None),
        text: text![TakeMana(N)],
        delegates: vec![on_activated(|g, _s, activated| {
            mutations::take_stored_mana(g, activated.card_id(), N, OnZeroStored::Sacrifice)
                .map(|_| ())
        })],
    }
}

/// Minion combat ability which deals damage to the Champion player during
/// combat, causing them to discard `N` random cards and lose the game if they
/// cannot.
pub fn combat_deal_damage<const N: DamageAmount>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: trigger_text(Combat, text![DealDamage(N)]),
        delegates: vec![combat(|g, s, _| mutations::deal_damage(g, s, N))],
    }
}

/// Minion combat ability which ends the current raid in failure.
pub fn combat_end_raid() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: trigger_text(Combat, text!["End the raid"]),
        delegates: vec![combat(|g, _, _| mutations::end_raid(g, RaidOutcome::Failure))],
    }
}

/// Minion combat ability which gains mana
pub fn combat_gain_mana<const N: ManaValue>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: trigger_text(Combat, text![Gain, Mana(N)]),
        delegates: vec![combat(|g, _, _| {
            mana::gain(g, Side::Overlord, N);
            Ok(())
        })],
    }
}

/// Minion combat ability which causes the Champion player to lose action
/// points.
pub fn remove_actions_if_able<const N: ActionCount>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: trigger_text(Combat, text!["Remove", Actions(1)]),
        delegates: vec![combat(|g, _s, _| {
            mutations::lose_action_points_if_able(g, Side::Champion, N)
        })],
    }
}

/// An ability which allows a card to have level counters placed on it.
pub fn level_up() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text![LevelUp],
        delegates: vec![Delegate::CanLevelUpCard(QueryDelegate {
            requirement: this_card,
            transformation: |_g, _, _, current| current.with_override(true),
        })],
    }
}

pub fn construct() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text![Construct],
        delegates: vec![Delegate::MinionDefeated(EventDelegate {
            requirement: this_card,
            mutation: |g, s, _| {
                mutations::move_card(g, s.card_id(), CardPosition::DiscardPile(s.side()))
            },
        })],
    }
}

/// An [AbilityType] for an ability with "Sacrifice:" as its only cost.
pub fn sacrifice_this() -> AbilityType {
    AbilityType::Activated(
        Cost { mana: None, actions: 0, custom_cost: costs::sacrifice_cost() },
        TargetRequirement::None,
    )
}
