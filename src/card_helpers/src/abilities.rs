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
use game_data::card_state::{CardCounter, CardPosition};
use game_data::delegate_data::{Delegate, EventDelegate, Flag, QueryDelegate, RaidOutcome};
use game_data::game_history::{AbilityActivationType, HistoryEvent};
use game_data::game_updates::InitiatedBy;
use game_data::primitives::{AbilityId, DamageAmount, ManaValue};
use game_data::text::TextToken::*;
use rules::mutations::OnZeroStored;
use rules::{deal_damage, mutations};

use crate::text_macro::text;
use crate::this::on_activated;
use crate::*;

/// Helper to flatten a list of `Option` and remove `None` values.
pub fn some(abilities: Vec<Option<Ability>>) -> Vec<Ability> {
    abilities.into_iter().flatten().collect()
}

/// Returns the provided [Ability] only for upgraded versions of a card.
pub fn when_upgraded(metadata: CardMetadata, ability: Ability) -> Option<Ability> {
    metadata.is_upgraded.then_some(ability)
}

pub fn silent_ability(ability: Ability) -> Ability {
    Ability { text: text![], ..ability }
}

/// The standard weapon ability; applies an attack boost for the duration of a
/// single encounter.
pub fn encounter_boost() -> Ability {
    Ability::new(encounter_ability_text(text![EncounterBoostCost], text![EncounterBoostBonus]))
}

/// The standard weapon breach ability, reads the weapon's breach value from its
/// definition.
pub fn breach() -> Ability {
    Ability::new(text![Breach])
}

/// Store `N` mana in this card when played. Move it to the discard pile when
/// the stored mana is depleted.
pub fn store_mana_on_play<const N: ManaValue>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: trigger_text(Play, text![StoreMana(N)]),
        delegates: vec![
            Delegate::PlayCard(EventDelegate::new(this_card, |g, _s, played| {
                g.card_mut(played.card_id).set_counters(CardCounter::StoredMana, N);
                Ok(())
            })),
            Delegate::StoredManaTaken(EventDelegate::new(this_card, |g, s, card_id| {
                if g.card(*card_id).counters(CardCounter::StoredMana) == 0 {
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
        ability_type: AbilityType::Activated {
            cost,
            target_requirement: TargetRequirement::None,
            can_activate: None,
        },
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
        delegates: vec![combat(|g, s, _| deal_damage::apply(g, s, N))],
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

/// An [AbilityType] for an ability with "Sacrifice:" as its only cost.
pub fn sacrifice_this() -> AbilityType {
    AbilityType::Activated {
        cost: Cost { mana: None, actions: 0, custom_cost: costs::sacrifice_cost() },
        target_requirement: TargetRequirement::None,
        can_activate: None,
    }
}

pub fn encounter_ability_text(
    cost: Vec<TextElement>,
    effect: Vec<TextElement>,
) -> Vec<TextElement> {
    vec![TextElement::EncounterAbility { cost, effect }]
}

/// The "slow" ability, which doubles shield costs when using a weapon
pub fn slow() -> Ability {
    Ability::new_with_delegate(
        text![
            encounter_ability_text(text![EncounterBoostCost], text![EncounterBoostBonus]),
            text![Slow]
        ],
        delegates::shield_value(
            |_, s, info| info.weapon_id == Some(s.card_id()),
            |_, _, _, current| current * 2,
        ),
    )
}

/// Creates a silent 'can play?' ability, usually used to help prevent user
/// errors by playing cards when they make no sense (e.g. have no valid
/// targets).
pub fn silent_can_play(predicate: TransformationFn<CardId, Flag>) -> Ability {
    Ability::new_with_delegate(text![], this::can_play(predicate))
}

/// Text for cards which can only be played as a player's first action
pub fn play_as_first_action() -> Ability {
    fn is_game_action(event: &HistoryEvent) -> bool {
        match event {
            HistoryEvent::GainManaAction
            | HistoryEvent::DrawCardAction(_)
            | HistoryEvent::RemoveCurseAction
            | HistoryEvent::DispelEvocationAction => true,
            HistoryEvent::PlayCard(_, _, initiated_by)
                if *initiated_by == InitiatedBy::GameAction =>
            {
                true
            }
            HistoryEvent::ActivateAbility(_, _, activation)
                if *activation == AbilityActivationType::GameAction =>
            {
                true
            }
            HistoryEvent::RaidBegin(e) if e.data == InitiatedBy::GameAction => true,
            HistoryEvent::CardProgress(_, _, initiated_by)
                if *initiated_by == InitiatedBy::GameAction =>
            {
                true
            }
            _ => false,
        }
    }

    Ability::new_with_delegate(
        text!["Play as your first", TextToken::ActionSymbol],
        this::can_play(|g, _, _, current| {
            current.with_override(!history::current_turn(g).any(is_game_action))
        }),
    )
}
