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

use card_helpers::{abilities, costs, show_prompt, text, this};
use core_data::game_primitives::{CardType, Rarity, School, Side};
use game_data::card_definition::{Ability, CardConfig, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::game_actions::{ButtonPromptContext, PromptChoice};
use game_data::game_effect::GameEffect;
use rules::CardDefinitionExt;

pub fn equivalent_exchange(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EquivalentExchange,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::overlord_card(meta, "equivalent_exchange"),
        card_type: CardType::Ritual,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: abilities::some(vec![
            abilities::when_not_upgraded(meta, abilities::play_only_if_champion_cursed()),
            Some(abilities::silent_can_play(|g, _, _, current| {
                current.add_constraint(
                    g.score_area(Side::Overlord).filter(|c| c.definition().is_scheme()).count() > 0
                        && g.score_area(Side::Champion)
                            .filter(|c| c.definition().is_scheme())
                            .count()
                            > 0,
                )
            })),
            Some(Ability::new_with_delegate(
                text!["Swap a scheme in your score area with one in the Champion's score area"],
                this::on_played(|g, s, _| {
                    // Note that second option is shown first on prompt stack

                    show_prompt::with_context_and_choices(
                        g,
                        s,
                        ButtonPromptContext::CardToTakeFromOpponent,
                        g.score_area(Side::Champion)
                            .filter(|c| c.definition().is_scheme())
                            .map(|c| {
                                PromptChoice::new()
                                    .effect(GameEffect::SwapWithSelected(s.side(), c.id))
                                    .anchor_card(c.id)
                            })
                            .collect(),
                    );

                    show_prompt::with_context_and_choices(
                        g,
                        s,
                        ButtonPromptContext::CardToGiveToOpponent,
                        g.score_area(Side::Overlord)
                            .filter(|c| c.definition().is_scheme())
                            .map(|c| {
                                PromptChoice::new()
                                    .effect(GameEffect::SelectCardForPrompt(s.side(), c.id))
                                    .anchor_card(c.id)
                            })
                            .collect(),
                    );

                    Ok(())
                }),
            )),
        ]),
        config: CardConfig::default(),
    }
}
