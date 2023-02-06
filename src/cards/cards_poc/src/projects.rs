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

//! Card definitions for the Project card type

use assets::rexard_images;
use assets::rexard_images::RexardPack;
use card_helpers::{abilities, text, text2, *};
use game_data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition};
use game_data::card_name::CardName;
use game_data::card_set_name::CardSetName;
use game_data::primitives::{CardType, Rarity, School, Side};
use game_data::text::{Keyword, Sentence};
use game_data::text2::trigger;
use game_data::text2::Token::*;
use rules::mutations;
use rules::mutations::OnZeroStored;

pub fn gemcarver() -> CardDefinition {
    let t2 = text2![Unveil, "at", Dusk, "then", StoreMana(9)];
    let t3 = trigger(Dusk, text2![text2![TakeMana(3)], text2!["When empty, draw a card"]]);

    CardDefinition {
        name: CardName::Gemcarver,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(2),
        image: rexard_images::get(RexardPack::MiningIcons, "MiningIcons_30_b"),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability {
                text: text![Keyword::Unveil, "at Dusk, then", Keyword::Store(Sentence::Start, 9)],
                ability_type: AbilityType::Standard,
                delegates: vec![unveil_at_dusk(), store_mana_on_unveil::<9>()],
            },
            simple_ability(
                text![
                    Keyword::Dusk,
                    Keyword::Take(Sentence::Start, 3),
                    ".",
                    "When empty, draw a card."
                ],
                at_dusk(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)?;
                    if g.card(s.card_id()).data.stored_mana == 0 {
                        mutations::draw_cards(g, s.side(), 1)?;
                    }

                    // TODO: Consider not alerting on the first turn to avoid two popups
                    alert(g, s);
                    Ok(())
                }),
            ),
        ],
        config: CardConfig::default(),
    }
}

pub fn spike_trap() -> CardDefinition {
    let t2 = trigger(
        Trap,
        text2!["If this card is in play,", DealDamage(2), "plus", 1, "per level counter"],
    );

    CardDefinition {
        name: CardName::SpikeTrap,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(2),
        image: rexard_images::get(RexardPack::MiningIcons, "MiningIcons_45_b"),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::level_up(),
            simple_ability(
                text![
                    Keyword::Trap,
                    "If this card is in play, deal 2 damage plus 1 per level counter"
                ],
                on_accessed(|g, s, _| {
                    if g.card(s.card_id()).position().in_play() {
                        mutations::deal_damage(g, s, 2 + g.card(s.card_id()).data.card_level)?;
                        alert(g, s);
                    }

                    Ok(())
                }),
            ),
        ],
        config: CardConfig::default(),
    }
}
