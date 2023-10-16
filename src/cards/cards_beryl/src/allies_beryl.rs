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

use card_helpers::{abilities, costs, in_play, show_prompt, text, this};
use game_data::card_definition::{CardConfig, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::game_actions::{ButtonPromptContext, PromptChoice};
use game_data::game_effect::GameEffect;
use game_data::primitives::{CardSubtype, CardType, Rarity, School, Side};
use game_data::text::TextElement;
use game_data::text::TextToken::*;
use rules::curses;

pub fn stalwart_protector(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::StalwartProtector,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(1, 0)),
        image: assets::champion_card(meta, "stalwart_protector"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Warrior],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::standard(
                text![TextElement::Activated {
                    cost: text![SacrificeCost],
                    effect: text!["Prevent receiving a", Curse]
                }],
                in_play::on_will_receive_curses(|g, s, _| {
                    show_prompt::with_context_and_choices(
                        g,
                        s,
                        ButtonPromptContext::SacrificeToPreventCurses(s.card_id(), 1),
                        vec![
                            PromptChoice::new()
                                .effect(GameEffect::SacrificeCard(s.card_id()))
                                .effect(GameEffect::PreventCurses(1)),
                            PromptChoice::new().effect(GameEffect::Continue),
                        ],
                    );
                    Ok(())
                }),
            ),
            abilities::activated(
                text!["Remove a curse"],
                costs::sacrifice(),
                this::on_activated(|g, _, _| curses::remove_curses(g, 1)),
            ),
        ],
        config: CardConfig::default(),
    }
}
