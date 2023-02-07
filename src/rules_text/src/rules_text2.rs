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

use std::iter;

use core_ui::icons;
use core_ui::prelude::Node;
use game_data::card_definition::{Ability2, AbilityType, CardDefinition2, Cost};
use game_data::primitives::{AbilityId, AbilityIndex};
use game_data::text::RulesTextContext;
use game_data::text2::{Text2, Token};
use protos::spelldawn::RulesText;

/// Primary function which turns the current state of a card into its client
/// [RulesText] representation
pub fn build(_: &RulesTextContext, definition: &CardDefinition2) -> RulesText {
    let mut lines = vec![];
    for ability in &definition.abilities {
        if let AbilityType::Activated(cost, _) = &ability.ability_type {
            lines.push(build_text(
                &vec![Text2::Activated {
                    cost: ability_cost_string(cost),
                    effect: ability.text.clone(),
                }],
                true,
            ))
        } else {
            lines.push(build_text(&ability.text, true))
        }
    }

    if let Some(breach) = definition.config.stats.breach {
        lines.push(build_text(&vec![Text2::Token(Token::Breach(breach))], false))
    }

    RulesText { text: lines.join("\n") }
}

/// Builds the rules text for a single [Ability], not including its cost (if
/// any).
pub fn ability_text(_: &RulesTextContext, ability: &Ability2) -> String {
    build_text(&ability.text, true)
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
fn build_text(text: &[Text2], add_period: bool) -> String {
    let mut result = text.iter().map(process_text).collect::<Vec<_>>().join(" ");
    if add_period {
        match text[text.len() - 1] {
            Text2::Token(_) | Text2::Literal(_) | Text2::Reminder(_) => result.push_str("."),
            _ => {}
        }
    }
    result
}

fn process_text(text: &Text2) -> String {
    match text {
        Text2::Children(children) => build_text(children, true),
        Text2::NamedTrigger(token, children) => {
            format!(
                "{}<b>{}</b>: {}",
                icons::ACTION,
                process_token(token),
                build_text(children, true)
            )
        }
        Text2::Activated { cost, effect } => {
            format!("{} {} {}", build_text(cost, false), icons::ARROW, build_text(effect, true))
        }
        Text2::Literal(string) => string.clone(),
        Text2::Reminder(string) => string.clone(),
        Text2::Token(token) => process_token(token),
    }
}

fn process_token(token: &Token) -> String {
    match token {
        Token::ManaSymbol => icons::MANA.to_string(),
        Token::Mana(n) => format!("{n}{}", icons::MANA),
        Token::ActionSymbol => icons::ACTION.to_string(),
        Token::Actions(n) => format!("{n}{}", icons::ACTION),
        Token::Number(n) => n.to_string(),
        Token::Plus(n) => format!("+{n}"),
        Token::EncounterBoost => "<Encounter Boost>".to_string(),
        Token::Then => "then".to_string(),
        Token::Attack => "attack".to_string(),
        Token::Health => "health".to_string(),
        Token::Gain => "gain".to_string(),
        Token::Lose => "lose".to_string(),
        Token::Play => "play".to_string(),
        Token::Dawn => "dawn".to_string(),
        Token::Dusk => "dusk".to_string(),
        Token::Score => "score".to_string(),
        Token::Combat => "combat".to_string(),
        Token::Encounter => "encounter".to_string(),
        Token::Unveil => "unveil".to_string(),
        Token::BeginARaid => "begin a raid".to_string(),
        Token::SuccessfulRaid => "successful raid".to_string(),
        Token::StoreMana(n) => format!("<b>store</b> {n}"),
        Token::TakeMana(n) => format!("<b>take</b> {n}"),
        Token::DealDamage(n) => format!("deal {n} damage"),
        Token::TakeDamage(n) => format!("take {n} damage"),
        Token::InnerRoom => "inner room".to_string(),
        Token::OuterRoom => "outer room".to_string(),
        Token::Sanctum => "sanctum".to_string(),
        Token::Vault => "vault".to_string(),
        Token::Crypts => "crypts".to_string(),
        Token::Breach(n) => format!("<b>breach {n}</b>"),
        Token::LevelUp => "level up".to_string(),
        Token::Trap => "trap".to_string(),
        Token::Construct => "construct".to_string(),
    }
}
