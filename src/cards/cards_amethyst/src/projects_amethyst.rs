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
use card_helpers::{abilities, text, *};
use game_data::card_definition::{Ability, AbilityType, CardConfigBuilder, CardDefinition};
use game_data::card_name::CardName;
use game_data::card_set_name::CardSetName;
use game_data::primitives::{CardSubtype, CardType, Rarity, School, Side};
use rules::mutations;
use rules::mutations::OnZeroStored;

pub fn gemcarver() -> CardDefinition {
    CardDefinition {
        name: CardName::Gemcarver,
        sets: vec![CardSetName::Amethyst],
        cost: cost(2),
        image: rexard_images::get(RexardPack::MiningIcons, "MiningIcons_30_b"),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            projects::store_mana_on_unveil::<9>(),
            Ability {
                ability_type: AbilityType::Standard,
                text: trigger_text(
                    Dusk,
                    text![text![TakeMana(3)], text!["When empty, draw a card"]],
                ),
                delegates: vec![in_play::at_dusk(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)?;
                    if g.card(s.card_id()).data.stored_mana == 0 {
                        mutations::draw_cards(g, s.side(), 1)?;
                    }
                    alert(g, s);
                    Ok(())
                })],
            },
        ],
        config: CardConfigBuilder::new().raze_cost(2).build(),
    }
}

pub fn spike_trap() -> CardDefinition {
    CardDefinition {
        name: CardName::SpikeTrap,
        sets: vec![CardSetName::Amethyst],
        cost: cost(0),
        image: rexard_images::get(RexardPack::MiningIcons, "MiningIcons_45_b"),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Trap],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::level_up(),
            simple_ability(
                trigger_text(
                    Trap,
                    text!["If this card is in play,", DealDamage(2), "plus", 1, "per level"],
                ),
                on_accessed(|g, s, _| {
                    if g.card(s.card_id()).position().in_play() {
                        mutations::deal_damage(g, s, 2 + g.card(s.card_id()).data.card_level)?;
                        alert(g, s);
                    }

                    Ok(())
                }),
            ),
        ],
        config: CardConfigBuilder::new().raze_cost(2).build(),
    }
}
