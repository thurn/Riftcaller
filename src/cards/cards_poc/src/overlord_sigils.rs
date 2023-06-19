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

pub fn arcane_sigil() -> CardDefinition {
    CardDefinition {
        name: CardName::ArcaneSigil,
        sets: vec![CardSetName::ProofOfConcept],
        cost: sigil_cost(),
        image: rexard_images::spell(8, "SpellBook08_22"),
        card_type: CardType::Sigil,
        side: Side::Overlord,
        school: School::Shadow,
        rarity: Rarity::Exalted,
        abilities: vec![simple_ability(
            text!["The second spell you cast each turn does not cost", ActionSymbol],
            in_play::on_query_action_cost(|g, _, _, actions| {
                let cards = history::cards_played_this_turn(g);
                if cards.filter(|id| rules::card_definition(g, *id).is_spell()).count() == 1 {
                    0
                } else {
                    actions
                }
            }),
        )],
        config: CardConfig::default(),
    }
}
