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
use card_definition_data::card_view_context::CardViewContext;
use core_data::game_primitives::AbilityId;
use core_ui::icons;
use dispatcher::dispatch;
use game_data::card_configuration::{AttackBoost, Cost};
use game_data::delegate_data::{CardStatusMarker, IsSlowWeaponQuery};
use game_data::text::{TextElement, TextToken};
use protos::riftcaller::RulesText;

pub mod card_icons;
pub mod card_info;
pub mod supplemental_info;

/// Primary function which turns the current state of a card into its client
/// [RulesText] representation
pub fn build(context: &CardViewContext) -> RulesText {
    let mut lines = vec![];
    let abilities = aggregate_named_triggers(&context.definition().abilities);
    for ability in abilities {
        let mut text = build_text(context, &ability, should_add_ability_period(&ability));
        text = text.replace(" ,", ",");
        text = text.replace(" .", ".");

        if !text.is_empty() {
            lines.push(text);
        }
    }

    RulesText { text: lines.join("\n") }
}

/// Merge together abilities which have the same trigger in order to conserve
/// space.
fn aggregate_named_triggers(abilities: &[Ability]) -> Vec<Vec<TextElement>> {
    let mut results: Vec<(Option<TextToken>, Vec<TextElement>)> = vec![];
    for ability in abilities {
        if let AbilityType::Activated { cost, .. } = &ability.ability_type {
            results.push((
                None,
                vec![TextElement::Activated {
                    cost: ability_cost_string(cost),
                    effect: ability.text.clone(),
                }],
            ));
        } else if ability.text.len() == 1 {
            if let all @ TextElement::NamedTrigger(name, text) = &ability.text[0] {
                let mut found = false;
                for (i, (token, existing)) in results.iter().enumerate() {
                    if Some(name) == token.as_ref() {
                        let mut new_text = existing.clone();
                        new_text.push(TextElement::Children(text.clone()));
                        results[i] = (*token, new_text);
                        found = true;
                        break;
                    }
                }

                if !found {
                    results.push((Some(*name), vec![all.clone()]));
                }
            } else {
                results.push((None, ability.text.clone()));
            }
        } else {
            results.push((None, ability.text.clone()));
        }
    }

    results.into_iter().map(|(_, elements)| elements).collect()
}

/// Builds the rules text for a single [Ability], not including its cost (if
/// any).
pub fn ability_text(context: &CardViewContext, ability: &Ability) -> String {
    if let AbilityType::Activated { cost, .. } = &ability.ability_type {
        let text = vec![TextElement::Activated {
            cost: ability_cost_string(cost),
            effect: ability.text.clone(),
        }];
        build_text(context, &text, true)
    } else {
        build_text(context, &ability.text, true)
    }
}

/// Builds text for a status marker card.
pub fn status_marker_text(context: &CardViewContext, status_marker: CardStatusMarker) -> String {
    build_text(context, &status_marker.text, true)
}

fn ability_cost_string(cost: &Cost<AbilityId>) -> Vec<TextElement> {
    let mut result = vec![];

    if cost.actions > 0 {
        result.push(TextElement::Token(TextToken::Actions(cost.actions)));
    }

    if let Some(mana) = cost.mana {
        if mana > 0 {
            result.push(TextElement::Token(TextToken::Mana(mana)));
        }
    }

    if let Some(description) =
        cost.custom_cost.as_ref().and_then(|custom| custom.description.as_ref())
    {
        if !result.is_empty() {
            result.push(TextElement::Literal(",".to_string()));
        }
        result.push(description.clone());
    }

    result
}

/// Combines a series of text elements together separated by the space
/// character.
///
/// If `add_period` is true, appends a final '.' if the last element is not
/// itself a sentence-level element.
fn build_text(context: &CardViewContext, text: &[TextElement], add_period: bool) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut result =
        text.iter().map(|text| process_text(context, text)).collect::<Vec<_>>().join(" ");
    if add_period {
        match text[text.len() - 1] {
            TextElement::Token(_) => result.push('.'),
            TextElement::Literal(_) | TextElement::Reminder(_) => result.push('.'),
            _ => {}
        }
    }
    capitalize(result)
}

fn capitalize(mut s: String) -> String {
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
    s
}

fn process_text(context: &CardViewContext, text: &TextElement) -> String {
    match text {
        TextElement::Children(children) => build_text(context, children, true),
        TextElement::NamedTrigger(token, children) => {
            format!(
                "{}{}: {}",
                icons::TRIGGER,
                process_token(context, token),
                build_text(context, children, true)
            )
        }
        TextElement::Activated { cost, effect } => {
            format!(
                "{}<b>:</b> {}",
                build_text(context, cost, false),
                build_text(context, effect, true)
            )
        }
        TextElement::EncounterAbility { cost, effect } => {
            format!("{}: {}", build_text(context, cost, false), build_text(context, effect, true))
        }
        TextElement::CardName(name) => name.displayed_name(),
        TextElement::Literal(string) => string.clone(),
        TextElement::Reminder(string) => format!("<i>{string}</i>"),
        TextElement::Token(token) => process_token(context, token),
    }
}

fn process_token(context: &CardViewContext, token: &TextToken) -> String {
    match token {
        TextToken::ManaSymbol => icons::MANA.to_string(),
        TextToken::Mana(n) => format!("{n}{}", icons::MANA),
        TextToken::GainMana(value) => {
            format!("gain{}{}{}", icons::NON_BREAKING_SPACE, value, icons::MANA)
        }
        TextToken::LosesMana(value) => {
            format!("loses{}{}{}", icons::NON_BREAKING_SPACE, value, icons::MANA)
        }
        TextToken::PayMana(value) => {
            format!("pay{}{}{}", icons::NON_BREAKING_SPACE, value, icons::MANA)
        }
        TextToken::ManaMinus(n) => format!("-{n}{}", icons::MANA),
        TextToken::ActionSymbol => icons::ACTION.to_string(),
        TextToken::Actions(n) => icons::ACTION.repeat(*n as usize),
        TextToken::GainActions(n) => {
            format!("gain{}{}", icons::NON_BREAKING_SPACE, icons::ACTION.repeat(*n as usize))
        }
        TextToken::PowerChargeSymbol => icons::POWER_CHARGE.to_string(),
        TextToken::PowerCharges(n) => {
            if *n == 1 {
                icons::POWER_CHARGE.to_string()
            } else {
                format!("{}{}", n, icons::POWER_CHARGE)
            }
        }
        TextToken::AddPowerCharges(n) => {
            if *n == 1 {
                format!("add{}{}", icons::NON_BREAKING_SPACE, icons::POWER_CHARGE)
            } else {
                format!("add{}{}{}", icons::NON_BREAKING_SPACE, n, icons::POWER_CHARGE)
            }
        }
        TextToken::Number(n) => n.to_string(),
        TextToken::Plus(n) => format!("+{n}"),
        TextToken::EncounterBoostCost => {
            format!("{}{}", encounter_boost(context).map_or(0, |boost| boost.cost), icons::MANA)
        }
        TextToken::EncounterBoostBonus => {
            format!("+{} attack", encounter_boost(context).map_or(0, |boost| boost.bonus))
        }
        TextToken::SacrificeCost => format!("{}Sacrifice", icons::TRIGGER),
        TextToken::Attack => "attack".to_string(),
        TextToken::Health => "health".to_string(),
        TextToken::Lose => "lose".to_string(),
        TextToken::Play => "Play".to_string(),
        TextToken::Dawn => "Dawn".to_string(),
        TextToken::Dusk => "Dusk".to_string(),
        TextToken::Score => "Score".to_string(),
        TextToken::Combat => "Combat".to_string(),
        TextToken::Encounter => "Encounter".to_string(),
        TextToken::BeginARaid => "Begin a raid".to_string(),
        TextToken::StoreMana(n) => format!("store {n}{}", icons::MANA),
        TextToken::TakeMana(n) => format!("take {n}{}", icons::MANA),
        TextToken::Damage => "damage".to_string(),
        TextToken::DealDamage(n) => format!("deal {n} damage"),
        TextToken::TakeDamage(n) => format!("take {n} damage"),
        TextToken::InnerRoom => "inner room".to_string(),
        TextToken::InnerRooms => "inner rooms".to_string(),
        TextToken::OuterRoom => "outer room".to_string(),
        TextToken::OuterRooms => "outer rooms".to_string(),
        TextToken::Sanctum => "sanctum".to_string(),
        TextToken::Vault => "vault".to_string(),
        TextToken::Crypt => "crypt".to_string(),
        TextToken::Breach => {
            format!("breach {}", context.definition().config.stats.breach.unwrap_or_default())
        }
        TextToken::CanProgress => "<b>Progress</b>".to_string(),
        TextToken::Trap => "<b>Trap</b>".to_string(),
        TextToken::Curse => "curse".to_string(),
        TextToken::Curses => "curses".to_string(),
        TextToken::Cursed => "cursed".to_string(),
        TextToken::SlowAbility => if context.query_id_or(true, |g, card_id| {
            dispatch::perform_query(g, IsSlowWeaponQuery(&card_id), false)
        }) {
            "slow"
        } else {
            "<s><i>Slow</i></s>"
        }
        .to_string(),
        TextToken::Mortal => assets::resonance_string("mortal"),
        TextToken::Infernal => assets::resonance_string("infernal"),
        TextToken::Astral => assets::resonance_string("astral"),
        TextToken::Prismatic => assets::resonance_string("prismatic"),
        TextToken::Wound => "wound".to_string(),
        TextToken::Leyline => "leyline".to_string(),
        TextToken::Leylines => "leylines".to_string(),
        TextToken::Evade => "evade".to_string(),
        TextToken::Evaded => "evaded".to_string(),
        TextToken::Evading => "evading".to_string(),
        TextToken::Unsummon => "unsummon".to_string(),
        TextToken::RazeAbility => format!("{} ability", icons::RAZE),
        TextToken::Banish => "banish".to_string(),
        TextToken::Permanent => "permanent".to_string(),
        TextToken::ShieldPoints => "shield points".to_string(),
    }
}

fn should_add_ability_period(text: &[TextElement]) -> bool {
    !(text.len() == 1 && matches!(text[0], TextElement::Token(_)))
}

fn encounter_boost<'a>(context: &'a CardViewContext) -> Option<&'a AttackBoost> {
    context.definition().config.stats.attack_boost.as_ref()
}
