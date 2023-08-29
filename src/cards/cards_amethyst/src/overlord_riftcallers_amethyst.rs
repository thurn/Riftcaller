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
use card_helpers::{history, *};
use game_data::card_definition::{CardConfig, CardDefinition};
use game_data::card_name::CardName;
use game_data::card_set_name::CardSetName;
use game_data::primitives::{CardType, Rarity, School, Side};

pub fn ubras_efaris_time_shaper() -> CardDefinition {
    CardDefinition {
        name: CardName::UbrasEfarisTimeShaper,
        sets: vec![CardSetName::Amethyst],
        cost: riftcaller_cost(),
        image: rexard_images::spell(8, "SpellBook08_22"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Shadow,
        rarity: Rarity::Exalted,
        abilities: vec![simple_ability(
            text!["The second spell you cast each turn does not cost", ActionSymbol],
            in_play::on_query_action_cost(|g, _, card_id, actions| {
                if rules::card_definition(g, *card_id).is_spell() {
                    let cards = history::cards_played_this_turn(g);
                    if cards.filter(|id| rules::card_definition(g, *id).is_spell()).count() == 1 {
                        return 0;
                    }
                }

                actions
            }),
        )],
        config: CardConfig::default(),
    }
}