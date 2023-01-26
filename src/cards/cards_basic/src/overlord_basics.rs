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

use assets::rexard_images::RexardPack;
use assets::{rexard_images, EnvironmentType};
use card_helpers::{text, *};
use data::card_definition::{
    Ability, AbilityType, CardConfig, CardDefinition, CardStats, SchemePoints, TargetRequirement,
};
use data::card_name::CardName;
use data::primitives::{CardType, Lineage, Rarity, School, Side};
use data::set_name::SetName;
use data::text::{DamageWord, Keyword, Sentence, TextToken};
use rules::mutations::OnZeroStored;
use rules::{mana, mutations};

pub fn basic_overlord_leader() -> CardDefinition {
    CardDefinition {
        name: CardName::BasicOverlordLeader,
        sets: vec![SetName::Basics],
        cost: leader_cost(),
        image: assets::fantasy_class_image("Warlock", "Male"),
        card_type: CardType::Leader,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![text_only_ability(text!["Your leader starts the game in play"])],
        config: CardConfig {
            player_portrait: Some(assets::fantasy_class_portrait(Side::Champion, "Warlock_M")),
            image_background: Some(assets::environments(
                EnvironmentType::CastlesTowersKeeps,
                "BoneKeep/SceneryBKeep_outside_2",
            )),
            ..CardConfig::default()
        },
    }
}

pub fn conspire() -> CardDefinition {
    CardDefinition {
        name: CardName::Conspire,
        sets: vec![SetName::Basics],
        cost: scheme_cost(),
        image: rexard_images::spell(2, "SpellBook02_17"),
        card_type: CardType::Scheme,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![],
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 3, points: 15 }),
            ..CardConfig::default()
        },
    }
}

pub fn devise() -> CardDefinition {
    CardDefinition {
        name: CardName::Devise,
        sets: vec![SetName::Basics],
        cost: scheme_cost(),
        image: rexard_images::spell(2, "SpellBook02_27"),
        card_type: CardType::Scheme,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![],
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 4, points: 30 }),
            ..CardConfig::default()
        },
    }
}

pub fn machinate() -> CardDefinition {
    CardDefinition {
        name: CardName::Machinate,
        sets: vec![SetName::Basics],
        cost: scheme_cost(),
        image: rexard_images::spell(2, "SpellBook02_29"),
        card_type: CardType::Scheme,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![],
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 5, points: 45 }),
            ..CardConfig::default()
        },
    }
}

pub fn gathering_dark() -> CardDefinition {
    CardDefinition {
        name: CardName::GatheringDark,
        sets: vec![SetName::Basics],
        cost: cost(5),
        image: rexard_images::spell(1, "SpellBook01_88"),
        card_type: CardType::OverlordSpell,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!("Gain", mana_text(9)),
            on_cast(|g, s, _| {
                mana::gain(g, s.side(), 9);
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn coinery() -> CardDefinition {
    CardDefinition {
        name: CardName::Coinery,
        sets: vec![SetName::Basics],
        cost: cost(2),
        image: rexard_images::get(RexardPack::LootIcons, "coins_b_03"),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![
            text_only_ability(text![
                Keyword::Unveil,
                "when activated, then",
                Keyword::Store(Sentence::Start, 15)
            ]),
            Ability {
                text: text![Keyword::Take(Sentence::Start, 3)],
                ability_type: AbilityType::Activated(actions(1), TargetRequirement::None),
                delegates: vec![
                    activate_while_face_down(),
                    face_down_ability_cost(),
                    on_activated(|g, s, _| {
                        if mutations::unveil_project_ignoring_costs(g, s.card_id())? {
                            add_stored_mana(g, s.card_id(), 15);
                        }
                        mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)
                            .map(|_| ())
                    }),
                ],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn leyline() -> CardDefinition {
    CardDefinition {
        name: CardName::Leyline,
        sets: vec![SetName::Basics],
        cost: cost(2),
        image: rexard_images::spell(2, "SpellBook02_78"),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![
            unveil_at_dusk_ability(),
            simple_ability(
                text![Keyword::Dusk, "Gain", TextToken::Mana(1)],
                at_dusk(|g, s, _| {
                    mana::gain(g, s.side(), 1);
                    Ok(())
                }),
            ),
        ],
        config: CardConfig::default(),
    }
}

pub fn ore_refinery() -> CardDefinition {
    CardDefinition {
        name: CardName::OreRefinery,
        sets: vec![SetName::Basics],
        cost: cost(4),
        image: rexard_images::get(RexardPack::MiningIcons, "MiningIcons_06_b"),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![
            Ability {
                text: text![
                    Keyword::Unveil,
                    "at Dusk, then",
                    Keyword::Store(Sentence::Internal, 12)
                ],
                ability_type: AbilityType::Standard,
                delegates: vec![unveil_at_dusk(), store_mana_on_unveil::<12>()],
            },
            simple_ability(
                text![Keyword::Dusk, Keyword::Take(Sentence::Start, 3)],
                at_dusk(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)?;
                    Ok(())
                }),
            ),
        ],
        config: CardConfig::default(),
    }
}

pub fn crab() -> CardDefinition {
    CardDefinition {
        name: CardName::Crab,
        sets: vec![SetName::Basics],
        cost: cost(4),
        image: rexard_images::get(RexardPack::MonstersAvatars, "64"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::end_raid()],
        config: CardConfig {
            stats: CardStats { health: Some(2), ..CardStats::default() },
            lineage: Some(Lineage::Infernal),
            ..CardConfig::default()
        },
    }
}

pub fn fire_goblin() -> CardDefinition {
    CardDefinition {
        name: CardName::FireGoblin,
        sets: vec![SetName::Basics],
        cost: cost(1),
        image: rexard_images::get(RexardPack::MonstersAvatars, "70"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text![
                Keyword::Combat,
                Keyword::DealDamage(DamageWord::DealStart, 1),
                ".",
                "Gain",
                mana_text(1),
                "."
            ],
            combat(|g, s, _| {
                mutations::deal_damage(g, s, 1)?;
                mana::gain(g, Side::Overlord, 1);
                Ok(())
            }),
        )],
        config: CardConfig {
            stats: CardStats { health: Some(1), shield: Some(2), ..CardStats::default() },
            lineage: Some(Lineage::Infernal),
            ..CardConfig::default()
        },
    }
}

pub fn toucan() -> CardDefinition {
    CardDefinition {
        name: CardName::Toucan,
        sets: vec![SetName::Basics],
        cost: cost(3),
        image: rexard_images::get(RexardPack::MonstersAvatars, "65"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::end_raid()],
        config: CardConfig {
            stats: CardStats { health: Some(3), ..CardStats::default() },
            lineage: Some(Lineage::Abyssal),
            ..CardConfig::default()
        },
    }
}

pub fn frog() -> CardDefinition {
    CardDefinition {
        name: CardName::Frog,
        sets: vec![SetName::Basics],
        cost: cost(4),
        image: rexard_images::get(RexardPack::MonstersAvatars, "66"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::end_raid()],
        config: CardConfig {
            stats: CardStats { health: Some(4), ..CardStats::default() },
            lineage: Some(Lineage::Abyssal),
            ..CardConfig::default()
        },
    }
}

pub fn captain() -> CardDefinition {
    CardDefinition {
        name: CardName::Captain,
        sets: vec![SetName::Basics],
        cost: cost(3),
        image: rexard_images::get(RexardPack::MonstersAvatars, "103"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::end_raid(), abilities::remove_actions_if_able::<1>()],
        config: CardConfig {
            stats: CardStats { health: Some(2), ..CardStats::default() },
            lineage: Some(Lineage::Mortal),
            ..CardConfig::default()
        },
    }
}

pub fn scout() -> CardDefinition {
    CardDefinition {
        name: CardName::Scout,
        sets: vec![SetName::Basics],
        cost: cost(5),
        image: rexard_images::get(RexardPack::MonstersAvatars, "19"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::end_raid()],
        config: CardConfig {
            stats: CardStats { health: Some(4), ..CardStats::default() },
            lineage: Some(Lineage::Mortal),
            ..CardConfig::default()
        },
    }
}
