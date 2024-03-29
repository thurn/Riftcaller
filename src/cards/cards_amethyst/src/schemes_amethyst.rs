// Copyright © Riftcaller 2021-present

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
use card_definition_data::ability_data::{Ability, AbilityType};
use card_definition_data::card_definition::CardDefinition;
use card_helpers::costs::scheme;
use card_helpers::text_helpers::named_trigger;
use card_helpers::this::on_scored_by_covenant;
use card_helpers::*;
use core_data::game_primitives::{CardType, Rarity, School, Side};
use game_data::card_configuration::{CardConfigBuilder, SchemePoints};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::delegate_data::{EventDelegate, GameDelegate, QueryDelegate};
use rules::mutations::SummonMinion;
use rules::{draw_cards, mana, mutations, queries};

pub fn gold_mine(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::GoldMine,
        sets: vec![CardSetName::Amethyst],
        cost: scheme(),
        image: rexard_images::get(RexardPack::MiningIcons, "MiningIcons_08_b"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: named_trigger(Score, text![GainMana(7)]),
            delegates: abilities::game(vec![on_scored_by_covenant(|g, s, _| {
                mana::gain(g, s.side(), 7);
                Ok(())
            })]),
        }],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 4, points: 20 })
            .build(),
    }
}

pub fn activate_reinforcements(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ActivateReinforcements,
        sets: vec![CardSetName::Amethyst],
        cost: scheme(),
        image: rexard_images::spell(1, "SpellBook01_01"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: text![
                "When this scheme is scored by either player, summon a face-down minion for free"
            ],
            delegates: abilities::game(vec![GameDelegate::ScoreCard(EventDelegate {
                requirement: this_card,
                mutation: |g, s, _| {
                    if let Some(minion_id) =
                        queries::highest_cost(g.minions().filter(|c| c.is_face_down()))
                    {
                        mutations::summon_minion(
                            g,
                            minion_id,
                            s.initiated_by(),
                            SummonMinion::IgnoreCosts,
                        )?;
                    }
                    Ok(())
                },
            })]),
        }],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 5, points: 30 })
            .build(),
    }
}

pub fn research_project(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ResearchProject,
        sets: vec![CardSetName::Amethyst],
        cost: scheme(),
        image: rexard_images::spell(1, "SpellBook01_03"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: named_trigger(
                Score,
                text![text!["Draw", 2, "cards"], text!["You get", Plus(2), "maximum hand size"]],
            ),
            delegates: abilities::game(vec![
                on_scored_by_covenant(|g, s, _| {
                    draw_cards::run(g, s.side(), 2, s.initiated_by()).map(|_| ())
                }),
                GameDelegate::MaximumHandSize(QueryDelegate {
                    requirement: scored_by_owner,
                    transformation: |_, s, side, current| {
                        if s.side() == *side {
                            current + 2
                        } else {
                            current
                        }
                    },
                }),
            ]),
        }],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 3, points: 10 })
            .build(),
    }
}
