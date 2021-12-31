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

//! Card definitions for the Scheme card type

use data::card_definition::{CardConfig, CardDefinition, SchemePoints};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};
use data::text::Keyword;

use crate::card_text::text;
use crate::helpers::*;
use crate::mutations;

pub fn dungeon_annex() -> CardDefinition {
    CardDefinition {
        name: CardName::DungeonAnnex,
        cost: cost(8),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_45"),
        card_type: CardType::Scheme,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![on_score(text![Keyword::Score, "Gain", mana(7)], |g, s, _| {
            mutations::gain_mana(g, s.side(), 7);
        })],
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 4, points: 2 }),
            ..CardConfig::default()
        },
    }
}
