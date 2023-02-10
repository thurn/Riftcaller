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

pub mod card_icons;
pub mod card_info;
pub mod supplemental_info;

use core_ui::icons;
use game_data::card_definition::{Ability, AbilityType, AttackBoost, Cost};
use game_data::card_view_context::CardViewContext;
use game_data::primitives::AbilityId;
use game_data::text::{TextElement, TextToken};
use protos::spelldawn::RulesText;
use rules::queries;

/// Primary function which turns the current state of a card into its client
/// [RulesText] representation
pub fn build(context: &CardViewContext) -> RulesText {
    let mut lines = vec![];
    let abilities = aggregate_named_triggers(&context.definition().abilities);
    for ability in abilities {
        let mut text = build_text(context, &ability, true);
        text = text.replace(" ,", ",");
        text = text.replace(" .", ".");

        if !text.is_empty() {
            lines.push(text);
        }
    }

    if let Some(breach) = context.definition().config.stats.breach {
        lines.push(build_text(context, &[TextElement::Token(TextToken::Breach(breach))], false))
    }

    RulesText { text: lines.join("\n") }
}

/// Merge together abilities which have the same trigger in order to conserve
/// space.
fn aggregate_named_triggers(abilities: &[Ability]) -> Vec<Vec<TextElement>> {
    let mut results: Vec<(Option<TextToken>, Vec<TextElement>)> = vec![];
    for ability in abilities {
        if let AbilityType::Activated(cost, _) = &ability.ability_type {
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
    build_text(context, &ability.text, true)
}

fn ability_cost_string(cost: &Cost<AbilityId>) -> Vec<TextElement> {
    let mut result = vec![];

    if cost.actions > 0 {
        result.push(TextElement::Token(TextToken::Actions(cost.actions)));
    }

    if let Some(mana) = cost.mana {
        if mana > 0 {
            result.push(TextElement::Token(TextToken::Mana(mana)))
        }
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
            TextElement::Token(t) if !(text.len() == 1 && t.is_keyword()) => result.push('.'),
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
                "{}<b>{}:</b> {}",
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
        TextElement::Literal(string) => string.clone(),
        TextElement::Reminder(string) => string.clone(),
        TextElement::Token(token) => process_token(context, token),
    }
}

fn process_token(context: &CardViewContext, token: &TextToken) -> String {
    match token {
        TextToken::ManaSymbol => icons::MANA.to_string(),
        TextToken::Mana(n) => format!("{n}{}", icons::MANA),
        TextToken::ManaMinus(n) => format!("-{n}{}", icons::MANA),
        TextToken::ActionSymbol => icons::ACTION.to_string(),
        TextToken::Actions(n) => icons::ACTION.repeat(*n as usize),
        TextToken::Number(n) => n.to_string(),
        TextToken::Plus(n) => format!("+{n}"),
        TextToken::EncounterBoostCost => {
            format!("{}{}", encounter_boost(context).cost, icons::MANA)
        }
        TextToken::EncounterBoostBonus => format!("+{} attack", encounter_boost(context).bonus),
        TextToken::Attack => "attack".to_string(),
        TextToken::Health => "health".to_string(),
        TextToken::Gain => "gain".to_string(),
        TextToken::Lose => "lose".to_string(),
        TextToken::Play => "Play".to_string(),
        TextToken::Dawn => "Dawn".to_string(),
        TextToken::Dusk => "Dusk".to_string(),
        TextToken::Score => "Score".to_string(),
        TextToken::Combat => "Combat".to_string(),
        TextToken::Encounter => "Encounter".to_string(),
        TextToken::Unveil => "<b>Unveil</b>".to_string(),
        TextToken::BeginARaid => "Begin a raid".to_string(),
        TextToken::StoreMana(n) => format!("<b>Store</b> {n}{}", icons::MANA),
        TextToken::TakeMana(n) => format!("<b>Take</b> {n}{}", icons::MANA),
        TextToken::DealDamage(n) => format!("deal {n} damage"),
        TextToken::TakeDamage(n) => format!("take {n} damage"),
        TextToken::InnerRoom => "inner room".to_string(),
        TextToken::OuterRoom => "outer room".to_string(),
        TextToken::Sanctum => "Sanctum".to_string(),
        TextToken::Vault => "Vault".to_string(),
        TextToken::Crypts => "Crypts".to_string(),
        TextToken::Breach(n) => format!("<b>Breach {n}</b>"),
        TextToken::LevelUp => "<b>Level Up</b>".to_string(),
        TextToken::Trap => "<b>Trap</b>".to_string(),
        TextToken::Construct => "<b>Construct</b>".to_string(),
    }
}

fn encounter_boost(context: &CardViewContext) -> AttackBoost {
    match context {
        CardViewContext::Default(definition) => definition.config.stats.attack_boost,
        CardViewContext::Game(_, game, card) => queries::attack_boost(game, card.id),
    }
    .unwrap_or_default()
}
