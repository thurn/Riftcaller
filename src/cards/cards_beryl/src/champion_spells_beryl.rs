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

use card_helpers::{card_predicates, costs, show_prompt, simple_ability, text, this};
use game_data::card_definition::{CardConfig, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::game_actions::PromptContext;
use game_data::primitives::{CardSubtype, CardType, Rarity, School, Side};

pub fn restoration(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Restoration,
        sets: vec![CardSetName::Amethyst],
        cost: costs::mana(1),
        image: assets::champion_card("restoration"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![CardSubtype::Conjuration],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!["Play an artifact in your discard pile"],
            this::on_cast(|g, s, _| {
                show_prompt::play_from_discard(
                    g,
                    s,
                    card_predicates::artifact,
                    PromptContext::PlayFromDiscard(CardType::Artifact),
                )
            }),
        )],
        config: CardConfig::default(),
    }
}
