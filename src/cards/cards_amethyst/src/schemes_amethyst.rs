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

use assets::rexard_images;
use assets::rexard_images::RexardPack;
use card_helpers::{text, *};
use game_data::card_definition::{
    Ability, AbilityType, CardConfigBuilder, CardDefinition, SchemePoints,
};
use game_data::card_name::CardName;
use game_data::card_set_name::CardSetName;
use game_data::delegates::{Delegate, EventDelegate, QueryDelegate};
use game_data::primitives::{CardType, Rarity, School, Side};
use rules::mutations::SummonMinion;
use rules::{mana, mutations, queries};

pub fn gold_mine() -> CardDefinition {
    CardDefinition {
        name: CardName::GoldMine,
        sets: vec![CardSetName::Amethyst],
        cost: scheme_cost(),
        image: rexard_images::get(RexardPack::MiningIcons, "MiningIcons_08_b"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: trigger_text(Score, text![Gain, Mana(7)]),
            delegates: vec![on_overlord_score(|g, s, _| {
                mana::gain(g, s.side(), 7);
                Ok(())
            })],
        }],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { level_requirement: 4, points: 30 })
            .build(),
    }
}

pub fn activate_reinforcements() -> CardDefinition {
    CardDefinition {
        name: CardName::ActivateReinforcements,
        sets: vec![CardSetName::Amethyst],
        cost: scheme_cost(),
        image: rexard_images::spell(1, "SpellBook01_01"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: text![
                "When this scheme is scored by either player, summon a face-down minion for free"
            ],
            delegates: vec![Delegate::ScoreCard(EventDelegate {
                requirement: this_card,
                mutation: |g, _, _| {
                    if let Some(minion_id) =
                        queries::highest_cost(g.minions().filter(|c| c.is_face_down()))
                    {
                        mutations::summon_minion(g, minion_id, SummonMinion::IgnoreCosts)?;
                    }
                    Ok(())
                },
            })],
        }],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { level_requirement: 5, points: 45 })
            .build(),
    }
}

pub fn research_project() -> CardDefinition {
    CardDefinition {
        name: CardName::ResearchProject,
        sets: vec![CardSetName::Amethyst],
        cost: scheme_cost(),
        image: rexard_images::spell(1, "SpellBook01_03"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: trigger_text(
                Score,
                text![text!["Draw", 2, "cards"], text!["You get", Plus(2), "maximum hand size"]],
            ),
            delegates: vec![
                on_overlord_score(|g, s, _| mutations::draw_cards(g, s.side(), 2).map(|_| ())),
                Delegate::MaximumHandSize(QueryDelegate {
                    requirement: scored_by_owner,
                    transformation: |_, s, side, current| {
                        if s.side() == *side {
                            current + 2
                        } else {
                            current
                        }
                    },
                }),
            ],
        }],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { level_requirement: 3, points: 15 })
            .build(),
    }
}
