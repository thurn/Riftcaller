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

use std::iter;

use core_ui::icons;
use core_ui::prelude::Node;
use game_data::card_definition::{Ability, AbilityType, AttackBoost, CardDefinition, Cost};
use game_data::primitives::{AbilityId, AbilityIndex};
use game_data::text2::{RulesTextContext, Text2, Token};
use protos::spelldawn::RulesText;
use rules::queries;

/// Primary function which turns the current state of a card into its client
/// [RulesText] representation
pub fn build(context: &RulesTextContext, definition: &CardDefinition) -> RulesText {
    let mut lines = vec![];
    for ability in &definition.abilities {
        let mut text = if let AbilityType::Activated(cost, _) = &ability.ability_type {
            build_text(
                context,
                &[Text2::Activated {
                    cost: ability_cost_string(cost),
                    effect: ability.text.clone(),
                }],
                true,
            )
        } else {
            build_text(context, &ability.text, true)
        };

        text = text.replace(" ,", ",");
        text = text.replace(" .", ".");

        if !text.is_empty() {
            lines.push(text);
        }
    }

    if let Some(breach) = definition.config.stats.breach {
        lines.push(build_text(context, &[Text2::Token(Token::Breach(breach))], false))
    }

    RulesText { text: lines.join("\n") }
}

/// Builds the rules text for a single [Ability], not including its cost (if
/// any).
pub fn ability_text(context: &RulesTextContext, ability: &Ability) -> String {
    build_text(context, &ability.text, true)
}

/// Builds the supplemental info display for a card, which displays additional
/// help information and appears on long-press.
///
/// If an `ability_index` is provided, only supplemental info for that index is
/// returned. Otherwise, supplemental info for all abilities is returned.
pub fn build_supplemental_info(_: &RulesTextContext, _: Option<AbilityIndex>) -> Option<Node> {
    None
}

fn ability_cost_string(cost: &Cost<AbilityId>) -> Vec<Text2> {
    let mut result = iter::repeat(Text2::Token(Token::ActionSymbol))
        .take(cost.actions as usize)
        .collect::<Vec<_>>();

    if let Some(mana) = cost.mana {
        if mana > 0 {
            result.push(Text2::Token(Token::Mana(mana)))
        }
    }

    result
}

/// Combines a series of text elements together separated by the space
/// character.
///
/// If `add_period` is true, appends a final '.' if the last element is not
/// itself a sentence-level element.
fn build_text(context: &RulesTextContext, text: &[Text2], add_period: bool) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut result =
        text.iter().map(|text| process_text(context, text)).collect::<Vec<_>>().join(" ");
    if add_period {
        match text[text.len() - 1] {
            Text2::Token(t) if text.len() > 1 || !t.is_keyword() => result.push('.'),
            Text2::Literal(_) | Text2::Reminder(_) => result.push('.'),
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

fn process_text(context: &RulesTextContext, text: &Text2) -> String {
    match text {
        Text2::Children(children) => build_text(context, children, true),
        Text2::NamedTrigger(token, children) => {
            format!(
                "{}<b>{}</b>: {}",
                icons::TRIGGER,
                process_token(context, token),
                build_text(context, children, true)
            )
        }
        Text2::Activated { cost, effect } => {
            format!(
                "{} {} {}",
                build_text(context, cost, false),
                icons::ARROW,
                build_text(context, effect, true)
            )
        }
        Text2::EncounterAbility { cost, effect } => {
            format!("{}: {}", build_text(context, cost, false), build_text(context, effect, true))
        }
        Text2::Literal(string) => string.clone(),
        Text2::Reminder(string) => string.clone(),
        Text2::Token(token) => process_token(context, token),
    }
}

fn process_token(context: &RulesTextContext, token: &Token) -> String {
    match token {
        Token::ManaSymbol => icons::MANA.to_string(),
        Token::Mana(n) => format!("{n}{}", icons::MANA),
        Token::ActionSymbol => icons::ACTION.to_string(),
        Token::Actions(n) => format!("{n}{}", icons::ACTION),
        Token::Number(n) => n.to_string(),
        Token::Plus(n) => format!("+{n}"),
        Token::EncounterBoostCost => format!("{}{}", encounter_boost(context).cost, icons::MANA),
        Token::EncounterBoostBonus => format!("+{} attack", encounter_boost(context).bonus),
        Token::Attack => "attack".to_string(),
        Token::Health => "health".to_string(),
        Token::Gain => "gain".to_string(),
        Token::Lose => "lose".to_string(),
        Token::Play => "Play".to_string(),
        Token::Dawn => "Dawn".to_string(),
        Token::Dusk => "Dusk".to_string(),
        Token::Score => "Score".to_string(),
        Token::Combat => "Combat".to_string(),
        Token::Encounter => "Encounter".to_string(),
        Token::Unveil => "<b>Unveil</b>".to_string(),
        Token::BeginARaid => "Begin a raid".to_string(),
        Token::SuccessfulRaid => "Successful Raid".to_string(),
        Token::StoreMana(n) => format!("<b>Store</b> {n}{}", icons::MANA),
        Token::TakeMana(n) => format!("<b>Take</b> {n}{}", icons::MANA),
        Token::DealDamage(n) => format!("deal {n} damage"),
        Token::TakeDamage(n) => format!("take {n} damage"),
        Token::InnerRoom => "inner room".to_string(),
        Token::OuterRoom => "outer room".to_string(),
        Token::Sanctum => "Sanctum".to_string(),
        Token::Vault => "Vault".to_string(),
        Token::Crypts => "Crypts".to_string(),
        Token::Breach(n) => format!("<b>Breach {n}</b>"),
        Token::LevelUp => "<b>Level Up</b>".to_string(),
        Token::Trap => "<b>Trap</b>".to_string(),
        Token::Construct => "<b>Construct</b>".to_string(),
    }
}

fn encounter_boost(context: &RulesTextContext) -> AttackBoost {
    match context {
        RulesTextContext::Default(definition) => definition.config.stats.attack_boost,
        RulesTextContext::Game(game, card) => queries::attack_boost(game, card.id),
    }
    .unwrap_or_default()
}
