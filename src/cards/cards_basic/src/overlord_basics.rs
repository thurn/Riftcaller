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
use assets::rexard_images::RexardPack;
use card_helpers::costs::scheme;
use card_helpers::this::on_activated;
use card_helpers::*;
use game_data::card_definition::{
    Ability, AbilityType, CardConfig, CardConfigBuilder, CardDefinition, Resonance, SchemePoints,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::primitives::{CardSubtype, CardType, Rarity, School, Side};
use rules::mutations::OnZeroStored;
use rules::{mana, mutations};

pub fn conspire(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Conspire,
        sets: vec![CardSetName::Basics],
        cost: scheme(),
        image: rexard_images::spell(2, "SpellBook02_17"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 3, points: 10 })
            .build(),
    }
}

pub fn devise(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Devise,
        sets: vec![CardSetName::Basics],
        cost: scheme(),
        image: rexard_images::spell(2, "SpellBook02_27"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 4, points: 20 })
            .build(),
    }
}

pub fn machinate(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Machinate,
        sets: vec![CardSetName::Basics],
        cost: scheme(),
        image: rexard_images::spell(2, "SpellBook02_29"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 5, points: 30 })
            .build(),
    }
}

pub fn gathering_dark(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::GatheringDark,
        sets: vec![CardSetName::Basics],
        cost: cost(5),
        image: rexard_images::spell(1, "SpellBook01_88"),
        card_type: CardType::OverlordSpell,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![GainMana(9)],
            this::on_played(|g, s, _| {
                mana::gain(g, s.side(), 9);
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn coinery(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Coinery,
        sets: vec![CardSetName::Basics],
        cost: cost(2),
        image: rexard_images::get(RexardPack::LootIcons, "coins_b_03"),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Nightbound],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![
            projects::store_mana_on_summon::<15>(),
            Ability {
                ability_type: activate_for_action(),
                text: text![TakeMana(3)],
                delegates: vec![on_activated(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)?;
                    Ok(())
                })],
            },
        ],
        config: CardConfigBuilder::new().raze_cost(3).build(),
    }
}

pub fn leyline(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Leyline,
        sets: vec![CardSetName::Basics],
        cost: cost(2),
        image: rexard_images::spell(2, "SpellBook02_78"),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: trigger_text(Dusk, text!["Gain", Mana(1)]),
            delegates: vec![in_play::at_dusk(|g, s, _| {
                mana::gain(g, s.side(), 1);
                Ok(())
            })],
        }],
        config: CardConfigBuilder::new().raze_cost(4).build(),
    }
}

pub fn ore_refinery(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::OreRefinery,
        sets: vec![CardSetName::Basics],
        cost: cost(4),
        image: rexard_images::get(RexardPack::MiningIcons, "MiningIcons_06_b"),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![
            projects::store_mana_on_summon::<12>(),
            Ability {
                ability_type: AbilityType::Standard,
                text: trigger_text(Dusk, text![TakeMana(3)]),
                delegates: vec![in_play::at_dusk(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)?;
                    Ok(())
                })],
            },
        ],
        config: CardConfigBuilder::new().raze_cost(4).build(),
    }
}

pub fn crab(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Crab,
        sets: vec![CardSetName::Basics],
        cost: cost(4),
        image: rexard_images::get(RexardPack::MonstersAvatars, "64"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::combat_end_raid()],
        config: CardConfigBuilder::new().health(2).resonance(Resonance::infernal()).build(),
    }
}

pub fn fire_goblin(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::FireGoblin,
        sets: vec![CardSetName::Basics],
        cost: cost(1),
        image: rexard_images::get(RexardPack::MonstersAvatars, "70"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::combat_deal_damage::<1>(), abilities::combat_gain_mana::<1>()],
        config: CardConfigBuilder::new()
            .health(1)
            .shield(2)
            .resonance(Resonance::infernal())
            .build(),
    }
}

pub fn toucan(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Toucan,
        sets: vec![CardSetName::Basics],
        cost: cost(3),
        image: rexard_images::get(RexardPack::MonstersAvatars, "65"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::combat_end_raid()],
        config: CardConfigBuilder::new().health(3).resonance(Resonance::astral()).build(),
    }
}

pub fn frog(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Frog,
        sets: vec![CardSetName::Basics],
        cost: cost(4),
        image: rexard_images::get(RexardPack::MonstersAvatars, "66"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::combat_end_raid()],
        config: CardConfigBuilder::new().health(4).resonance(Resonance::astral()).build(),
    }
}

pub fn captain(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Captain,
        sets: vec![CardSetName::Basics],
        cost: cost(3),
        image: rexard_images::get(RexardPack::MonstersAvatars, "103"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::combat_end_raid(), abilities::remove_actions_if_able::<1>()],
        config: CardConfigBuilder::new().health(2).resonance(Resonance::mortal()).build(),
    }
}

pub fn scout(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Scout,
        sets: vec![CardSetName::Basics],
        cost: cost(5),
        image: rexard_images::get(RexardPack::MonstersAvatars, "19"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::combat_end_raid()],
        config: CardConfigBuilder::new().health(4).resonance(Resonance::mortal()).build(),
    }
}
