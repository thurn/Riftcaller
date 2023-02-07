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

use core_ui::design::{self, FontColor};
use core_ui::icons;
use core_ui::prelude::*;
use game_data::card_definition::CardDefinition;
use game_data::primitives::{AbilityIndex, CardSubtype, CardType, Lineage};
use game_data::text::{RulesTextContext, TextElement, TextTokenKind};
use prompts::card_info::SupplementalCardInfo;

/// Builds the supplemental info display for a card, which displays additional
/// help information and appears on press.
///
/// If an `ability_index` is provided, only supplemental info for that index is
/// returned. Otherwise, supplemental info for all abilities is returned.
pub fn build(context: &RulesTextContext, ability_index: Option<AbilityIndex>) -> Option<Node> {
    let definition = rules::get(context.card_name());
    let mut result = vec![card_type_line(definition)];
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

    SupplementalCardInfo::new(result).build()
}

fn card_type_line(definition: &CardDefinition) -> String {
    let mut result = String::new();
    result.push_str(match definition.card_type {
        CardType::ChampionSpell => "Spell",
        CardType::Weapon => "Weapon",
        CardType::Artifact => "Artifact",
        CardType::OverlordSpell => "Spell",
        CardType::Minion => "Minion",
        CardType::Project => "Project",
        CardType::Scheme => "Scheme",
        CardType::Leader => "Leader",
        CardType::GameModifier => "Modifier",
    });

    if let Some(lineage) = definition.config.lineage {
        result.push_str(" • ");
        let (lineage, color) = match lineage {
            Lineage::Prismatic => ("Prismatic", FontColor::PrismaticCardTitle),
            Lineage::Construct => ("Construct", FontColor::ConstructCardTitle),
            Lineage::Mortal => ("Mortal", FontColor::MortalCardTitle),
            Lineage::Abyssal => ("Abyssal", FontColor::AbyssalCardTitle),
            Lineage::Infernal => ("Infernal", FontColor::InfernalCardTitle),
        };
        let string = format!("<color={}>{}</color>", design::as_hex(color), lineage);
        result.push_str(&string);
    }

    for subtype in &definition.config.subtypes {
        result.push_str(" • ");
        result.push_str(match subtype {
            CardSubtype::Silvered => "Silvered",
        });
    }

    result
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
        TextTokenKind::Unveil => entry("Unveil", "Pay cost and turn face up (if able)"),
        TextTokenKind::StoreMana => {
            entry("Store", format!("Add {} to this card to <b>take</b> later", icons::MANA))
        }
        TextTokenKind::DealDamage => {
            entry("Damage", "Causes the Champion to discard cards at random")
        }
        TextTokenKind::InnerRoom => entry("Inner Room", "The Sanctum, Vault or Crypts"),
        TextTokenKind::OuterRoom => {
            entry("Outer Room", "Room other than the Sanctum, Vault or Crypts")
        }
        TextTokenKind::Breach => {
            entry("Breach", "Allows this weapon to bypass some amount of Shield")
        }
        TextTokenKind::LevelUp => {
            entry("Level Up", "This card gets levels when its room is leveled up")
        }
        TextTokenKind::Trap => entry("Trap", "Triggers when this card is accessed during a raid"),
        TextTokenKind::Construct => {
            entry("Construct", "Goes to discard pile when defeated. Damage with any weapon.")
        }
        _ => None,
    }
}

fn entry(name: impl Into<String>, description: impl Into<String>) -> Option<String> {
    Some(format!("<b>{}</b>: {}.", name.into(), description.into()))
}
