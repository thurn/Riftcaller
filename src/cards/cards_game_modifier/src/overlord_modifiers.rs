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

use assets::rexard_images;
use card_helpers::{text, *};
use data::card_definition::{CardConfig, CardDefinition, Cost};
use data::card_name::CardName;
use data::delegates::{Delegate, QueryDelegate};
use data::primitives::{CardType, Rarity, School, Side};
use data::set_name::SetName;

pub fn tutorial_disable_draw_action() -> CardDefinition {
    CardDefinition {
        name: CardName::TutorialDisableDrawAction,
        sets: vec![SetName::TutorialEffects],
        cost: Cost::default(),
        image: rexard_images::spell(1, "SpellBook01_01"),
        card_type: CardType::GameModifier,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!["The Champion cannot take the 'draw card' action"],
            Delegate::CanTakeDrawCardAction(QueryDelegate {
                requirement: side_is_champion,
                transformation: |_, _, _, f| f.with_override(false),
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn tutorial_disable_gain_mana() -> CardDefinition {
    CardDefinition {
        name: CardName::TutorialDisableGainMana,
        sets: vec![SetName::TutorialEffects],
        cost: Cost::default(),
        image: rexard_images::spell(1, "SpellBook01_01"),
        card_type: CardType::GameModifier,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!["The Champion cannot take the 'gain mana' action"],
            Delegate::CanTakeGainManaAction(QueryDelegate {
                requirement: side_is_champion,
                transformation: |_, _, _, f| f.with_override(false),
            }),
        )],
        config: CardConfig::default(),
    }
}
