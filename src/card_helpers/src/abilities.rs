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

//! Helpers for defining common card abilities

use core_data::game_primitives::{AbilityId, InitiatedBy, ManaValue, INNER_ROOMS};
use game_data::card_definition::{Ability, AbilityType, Cost, TargetRequirement};
use game_data::card_name::CardMetadata;
use game_data::card_state::{CardCounter, CardPosition};
use game_data::custom_card_state::CustomCardState;
use game_data::delegate_data::{Delegate, EventDelegate, QueryDelegate};
use game_data::flag_data::Flag;
use game_data::game_actions::CardTarget;
use game_data::history_data::{AbilityActivationType, HistoryEvent};
use game_data::prompt_data::PromptData;
use game_data::text::TextToken::*;
use rules::mutations::OnZeroStored;
use rules::{curses, mana, mutations, prompts};

use crate::text_macro::text;
use crate::*;

/// Helper to flatten a list of `Option` and remove `None` values.
pub fn some(abilities: Vec<Option<Ability>>) -> Vec<Ability> {
    abilities.into_iter().flatten().collect()
}

/// Returns the provided [Ability] only for upgraded versions of a card.
pub fn when_upgraded(metadata: CardMetadata, ability: Ability) -> Option<Ability> {
    metadata.is_upgraded.then_some(ability)
}

/// Returns the provided [Ability] only for non-upgraded versions of a card.
pub fn when_not_upgraded(metadata: CardMetadata, ability: Ability) -> Option<Ability> {
    (!metadata.is_upgraded).then_some(ability)
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

/// Ability to gain mana when a card is played.
pub fn gain_mana_on_play<const N: ManaValue>() -> Ability {
    Ability::new_with_delegate(
        text![GainMana(N)],
        this::on_played(|g, s, _| {
            mana::gain(g, s.side(), N);
            Ok(())
        }),
    )
}

/// Store `N` mana in this card when played.
pub fn store_mana_on_play<const N: ManaValue>() -> Ability {
    Ability::new_with_delegate(
        text_helpers::named_trigger(Play, text![StoreMana(N)]),
        Delegate::PlayCard(EventDelegate::new(this_card, |g, _s, played| {
            g.card_mut(played.card_id).set_counters(CardCounter::StoredMana, N);
            Ok(())
        })),
    )
}

/// Store `N` mana in this card when played. Move it to the discard pile when
/// the stored mana is depleted.
pub fn store_mana_on_play_discard_on_empty<const N: ManaValue>() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text_helpers::named_trigger(Play, text![StoreMana(N)]),
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
        ability_type: AbilityType::Activated { cost, target_requirement: TargetRequirement::None },
        text: text![TakeMana(N)],
        delegates: vec![this::on_activated(|g, _s, activated| {
            mutations::take_stored_mana(g, activated.card_id(), N, OnZeroStored::Sacrifice)
                .map(|_| ())
        })],
    }
}

/// An ability which allows a card to have progress counters placed on it.
pub fn can_progress() -> Ability {
    Ability {
        ability_type: AbilityType::Standard,
        text: text![CanProgress],
        delegates: vec![Delegate::CanProgressCard(QueryDelegate {
            requirement: this_card,
            transformation: delegates::allow,
        })],
    }
}

/// An [AbilityType] for an ability with "Sacrifice:" as its only cost.
pub fn sacrifice_this() -> AbilityType {
    AbilityType::Activated {
        cost: Cost { mana: None, actions: 0, custom_cost: costs::sacrifice_custom_cost() },
        target_requirement: TargetRequirement::None,
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
    Ability::new_with_delegate(text![SlowAbility], this::is_slow_weapon(|_, _, _, _| true))
}

/// Creates a silent 'can play?' ability, usually used to help prevent user
/// errors by playing cards when they make no sense (e.g. have no valid
/// targets).
pub fn silent_can_play(predicate: TransformationFn<CardId, Flag>) -> Ability {
    Ability::new_with_delegate(text![], this::can_play(predicate))
}

/// Ability for cards which can only be played as a player's first action
pub fn play_as_first_action() -> Ability {
    fn is_game_action(event: &HistoryEvent) -> bool {
        match event {
            HistoryEvent::GainManaAction
            | HistoryEvent::DrawCardAction(..)
            | HistoryEvent::CardProgressAction(..)
            | HistoryEvent::RemoveCurseAction
            | HistoryEvent::DispelEvocationAction => true,
            HistoryEvent::PlayCard(_, _, initiated_by)
                if *initiated_by == InitiatedBy::GameAction =>
            {
                true
            }
            HistoryEvent::ActivateAbility(activation)
                if activation.activation_type == AbilityActivationType::GameAction =>
            {
                true
            }
            HistoryEvent::RaidBegin(e) if e.data == InitiatedBy::GameAction => true,
            _ => false,
        }
    }

    Ability::new_with_delegate(
        text!["Play as your first", TextToken::ActionSymbol],
        this::can_play(|g, _, _, current| {
            current.add_constraint(!history::current_turn(g).any(is_game_action))
        }),
    )
}

/// Ability for cards that can only be played if the Riftcaller is cursed
pub fn play_only_if_riftcaller_cursed() -> Ability {
    Ability::new_with_delegate(
        text!["Play only if the Riftcaller is", Cursed],
        this::can_play(|g, _, _, current| current.add_constraint(curses::get(g) > 0)),
    )
}

/// Ability for cards that can only be played if the sanctum, vault, and crypt
/// have been accessed this turn.
pub fn play_if_accessed_all_inner_rooms_this_turn() -> Ability {
    Ability::new_with_delegate(
        text!["Play only if you accessed the", Sanctum, ",", Vault, ", and", Crypt, "this turn"],
        this::can_play(|g, _, _, current| {
            current.add_constraint(
                INNER_ROOMS
                    .iter()
                    .all(|room| history::rooms_accessed_this_turn(g).any(|r| r == *room)),
            )
        }),
    )
}

/// Ability to choose a minion on play and store that choice for later.
pub fn choose_a_minion_in_target_room() -> Ability {
    Ability::new(text![TextElement::NamedTrigger(Play, text!["Choose a minion in target room"])])
        .delegate(this::on_played(|g, s, played| {
            prompts::push_with_data(g, s.side(), s, PromptData::CardPlay(*played));
            Ok(())
        }))
        .delegate(this::prompt(|g, s, source, _| {
            let PromptData::CardPlay(played) = source.data else {
                return None;
            };
            let CardTarget::Room(room_id) = played.target else {
                return None;
            };
            show_prompt::with_choices(
                g.defenders_unordered(room_id)
                    .map(|card| {
                        PromptChoice::new()
                            .effect(GameEffect::AppendCustomCardState(
                                s.card_id(),
                                CustomCardState::TargetCard {
                                    target_card: card.id,
                                    play_id: played.card_play_id,
                                },
                            ))
                            .anchor_card(card.id)
                    })
                    .collect(),
            )
        }))
}

/// Adds 1 attack to a weapon per power charge counter on it.
pub fn plus_1_attack_per_power_charge() -> Ability {
    Ability::new_with_delegate(
        text![Plus(1), "attack per", PowerCharges(1)],
        this::base_attack(|g, _, card_id, current| {
            current + g.card(*card_id).counters(CardCounter::PowerCharges)
        }),
    )
}
