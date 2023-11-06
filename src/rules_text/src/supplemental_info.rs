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

use convert_case::{Case, Casing};
use core_ui::icons;
use core_ui::prelude::*;
use game_data::card_definition::CardDefinition;
use game_data::card_view_context::CardViewContext;
use game_data::primitives::AbilityIndex;
use game_data::text::{TextElement, TextTokenKind};
use rules::queries;

use crate::card_info::SupplementalCardInfo;

/// Builds the supplemental info display for a card, which displays additional
/// help information and appears on press.
///
/// If an `ability_index` is provided, only supplemental info for that index is
/// returned. Otherwise, supplemental info for all abilities is returned.
pub fn build(context: &CardViewContext, ability_index: Option<AbilityIndex>) -> Option<Box<Node>> {
    let definition = context.definition();
    let mut result = vec![];
    if ability_index.is_none() {
        add_card_type_line(&mut result, context, definition);
    } else {
        result.push("Activated Ability".to_string())
    }

    let mut tokens = vec![];
    for (index, ability) in definition.abilities.iter().enumerate() {
        if matches!(ability_index, Some(i) if i.value() != index) {
            continue;
        }

        add_tokens(&mut tokens, &ability.text);
    }

    if definition.config.stats.breach.is_some() {
        tokens.push(TextTokenKind::Breach);
    }

    tokens.sort();
    tokens.dedup();
    result.extend(tokens.into_iter().filter_map(token_description));

    SupplementalCardInfo::new(result).build().map(Box::new)
}

fn add_card_type_line(
    builder: &mut Vec<String>,
    context: &CardViewContext,
    definition: &CardDefinition,
) {
    let mut result = String::new();
    result.push_str(&definition.card_type.to_string());
    let resonance = context.query_id_or(definition.config.resonance, queries::resonance);

    if let Some(resonance) = resonance {
        if resonance.mortal {
            append_resonance(&mut result, "Mortal");
        }
        if resonance.infernal {
            append_resonance(&mut result, "Infernal");
        }
        if resonance.astral {
            append_resonance(&mut result, "Astral");
        }
        if resonance.prismatic {
            append_resonance(&mut result, "Prismatic");
        }
    }

    for subtype in &definition.subtypes {
        result.push_str(" • ");
        result.push_str(&format!("{subtype}").from_case(Case::Pascal).to_case(Case::Title));
    }

    builder.push(result);
}

fn append_resonance(result: &mut String, name: &'static str) {
    result.push_str(" • ");
    result.push_str(&assets::resonance_string(name));
}

fn add_tokens(tokens: &mut Vec<TextTokenKind>, text: &[TextElement]) {
    for element in text {
        match element {
            TextElement::Token(t) => tokens.push(t.kind()),
            TextElement::Children(children) => add_tokens(tokens, children),
            TextElement::NamedTrigger(name, effect) => {
                tokens.push(name.kind());
                add_tokens(tokens, effect)
            }
            TextElement::Activated { cost, effect } => {
                add_tokens(tokens, cost);
                add_tokens(tokens, effect)
            }
            TextElement::EncounterAbility { cost, effect } => {
                add_tokens(tokens, cost);
                add_tokens(tokens, effect)
            }
            TextElement::Literal(_) => {}
            TextElement::Reminder(_) => {}
        }
    }
}

fn token_description(token: TextTokenKind) -> Option<String> {
    match token {
        TextTokenKind::PowerCharges => Some(format!(
            "{}: A power charge, stored while in play to spend on abilities", 
            icons::POWER_CHARGE
        )),
        TextTokenKind::Play => entry("Play", "Triggers when this card enters the arena"),
        TextTokenKind::Dawn => entry("Dawn", "Triggers at the start of the Champion's turn"),
        TextTokenKind::Dusk => entry("Dusk", "Triggers at the start of the Overlord's turn"),
        TextTokenKind::Score => entry("Score", "Triggers when the Overlord scores this card"),
        TextTokenKind::Combat => {
            entry("Combat", "Triggers if this minion is not defeated during a raid")
        }
        TextTokenKind::Encounter => {
            entry("Encounter", "Triggers when this minion is approached during a raid")
        }
        TextTokenKind::Unveil => entry("Unveil", "Triggers when this card is turned face up"),
        TextTokenKind::StoreMana => {
            entry("Store", format!("Add {} to this card to take later", icons::MANA))
        }
        TextTokenKind::Damage | TextTokenKind::DealDamage => {
            entry("Damage", "Causes the Champion to discard cards at random")
        }
        TextTokenKind::InnerRoom | TextTokenKind::InnerRooms => entry("Inner Room", "The sanctum, vault or crypt"),
        TextTokenKind::OuterRoom | TextTokenKind::OuterRooms => {
            entry("Outer Room", "Room other than the sanctum, vault or crypts")
        }
        TextTokenKind::Breach => {
            entry("Breach", "Allows this weapon to bypass some amount of Shield")
        }
        TextTokenKind::LevelUp => {
            entry("Level Up", "This card gets levels when its room is leveled up")
        }
        TextTokenKind::Trap => entry("Trap", "Triggers when this card is accessed during a raid"),
        TextTokenKind::Curse | TextTokenKind::Cursed => entry(
            "Curse",
            format!(
                "Allows Overlord to pay {} and 2{} to destroy evocations. Can be removed for {} and 2{}",
                icons::ACTION,
                icons::MANA,
                icons::ACTION,
                icons::MANA
            ),
        ),
        TextTokenKind::Slow => entry("Slow", "Shield costs are doubled when using this weapon"),
        TextTokenKind::Wound => entry("Wound", "Reduces your maximum hand size by 1"),
        TextTokenKind::Evade => entry("Evade", "Bypass a minion without combat"),
        TextTokenKind::Unsummon => entry("Unsummon", "Turn a minion face-down"),
        _ => None,
    }
}

fn entry(name: impl Into<String>, description: impl Into<String>) -> Option<String> {
    Some(format!("<b>{}</b>: {}.", name.into(), description.into()))
}
