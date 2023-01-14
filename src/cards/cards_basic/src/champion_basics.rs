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

use assets::rexard_images::{self, RexardPack, RexardWeaponType};
use card_helpers::{text, *};
use data::card_definition::{
    Ability, AbilityType, AttackBoost, CardConfig, CardDefinition, CardStats, TargetRequirement,
};
use data::card_name::CardName;
use data::primitives::{CardType, Lineage, Rarity, School, Side};
use data::set_name::SetName;
use data::special_effects::Projectile;
use data::text::{Keyword, Sentence};
use rules::mutations::OnZeroStored;
use rules::{mana, mutations};

pub fn basic_champion_identity() -> CardDefinition {
    CardDefinition {
        name: CardName::BasicChampionIdentity,
        sets: vec![SetName::Basics],
        cost: identity_cost(),
        image: rexard_images::get(RexardPack::MonstersAvatars, "22"),
        card_type: CardType::Identity,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![],
        config: CardConfig::default(),
    }
}

pub fn arcane_recovery() -> CardDefinition {
    CardDefinition {
        name: CardName::ArcaneRecovery,
        sets: vec![SetName::Basics],
        cost: cost(5),
        image: rexard_images::spell(1, "SpellBook01_24"),
        card_type: CardType::ChampionSpell,
        side: Side::Champion,
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

pub fn eldritch_surge() -> CardDefinition {
    CardDefinition {
        name: CardName::EldritchSurge,
        sets: vec![SetName::Basics],
        cost: cost(0),
        image: rexard_images::spell(1, "SpellBook01_56"),
        card_type: CardType::ChampionSpell,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!("Gain", mana_text(3)),
            on_cast(|g, s, _| {
                mana::gain(g, s.side(), 3);
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn lodestone() -> CardDefinition {
    CardDefinition {
        name: CardName::Lodestone,
        sets: vec![SetName::Basics],
        cost: cost(1),
        image: rexard_images::get(RexardPack::MagicItems, "orb_04_b"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana_on_play::<12>(),
            abilities::activated_take_mana::<2>(actions(1)),
        ],
        config: CardConfig::default(),
    }
}

pub fn mana_battery() -> CardDefinition {
    CardDefinition {
        name: CardName::ManaBattery,
        sets: vec![SetName::Basics],
        cost: cost(0),
        image: rexard_images::get(RexardPack::MagicItems, "artifact_11_b"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![
            simple_ability(
                text![Keyword::Dawn, Keyword::Take(Sentence::Start, 1)],
                at_dawn(|g, s, _| {
                    let taken =
                        mutations::take_stored_mana(g, s.card_id(), 1, OnZeroStored::Ignore)?;
                    alert_if_nonzero(g, s, taken);
                    Ok(())
                }),
            ),
            Ability {
                text: text![Keyword::Store(Sentence::Start, 3)],
                ability_type: AbilityType::Activated(actions(1), TargetRequirement::None),
                delegates: vec![on_activated(|g, s, _| {
                    add_stored_mana(g, s.card_id(), 3);
                    Ok(())
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn contemplate() -> CardDefinition {
    CardDefinition {
        name: CardName::Contemplate,
        sets: vec![SetName::Basics],
        cost: cost(0),
        image: rexard_images::spell(2, "SpellBook02_01"),
        card_type: CardType::ChampionSpell,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!("Gain", mana_text(2), ". Draw a card."),
            on_cast(|g, s, _| {
                mana::gain(g, s.side(), 2);
                mutations::draw_cards(g, s.side(), 1)?;
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn ancestral_knowledge() -> CardDefinition {
    CardDefinition {
        name: CardName::AncestralKnowledge,
        sets: vec![SetName::Basics],
        cost: cost(1),
        image: rexard_images::spell(3, "SpellBook03_46"),
        card_type: CardType::ChampionSpell,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!("Draw 3 cards."),
            on_cast(|g, s, _| {
                mutations::draw_cards(g, s.side(), 3)?;
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn simple_blade() -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleBlade,
        sets: vec![SetName::Basics],
        cost: cost(4),
        image: rexard_images::weapon(RexardWeaponType::Swords, "swnb_01"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: CardStats {
                base_attack: Some(2),
                attack_boost: Some(AttackBoost { cost: 1, bonus: 1 }),
                ..CardStats::default()
            },
            lineage: Some(Lineage::Mortal),
            special_effects: projectile(Projectile::Hovl(2)),
            ..CardConfig::default()
        },
    }
}

pub fn simple_axe() -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleAxe,
        sets: vec![SetName::Basics],
        cost: cost(4),
        image: rexard_images::weapon(RexardWeaponType::Axes, "a_n_b_01"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: CardStats {
                base_attack: Some(3),
                attack_boost: Some(AttackBoost { cost: 3, bonus: 1 }),
                ..CardStats::default()
            },
            lineage: Some(Lineage::Mortal),
            special_effects: projectile(Projectile::Hovl(2)),
            ..CardConfig::default()
        },
    }
}

pub fn simple_bow() -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleBow,
        sets: vec![SetName::Basics],
        cost: cost(0),
        image: rexard_images::weapon(RexardWeaponType::Bows, "b_b_02"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: CardStats {
                base_attack: Some(1),
                attack_boost: Some(AttackBoost { cost: 2, bonus: 1 }),
                ..CardStats::default()
            },
            lineage: Some(Lineage::Abyssal),
            special_effects: projectile(Projectile::Hovl(3)),
            ..CardConfig::default()
        },
    }
}

pub fn simple_club() -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleClub,
        sets: vec![SetName::Basics],
        cost: cost(2),
        image: rexard_images::weapon(RexardWeaponType::Clubs, "bl_b_07"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: CardStats {
                base_attack: Some(2),
                attack_boost: Some(AttackBoost { cost: 1, bonus: 1 }),
                ..CardStats::default()
            },
            lineage: Some(Lineage::Abyssal),
            special_effects: projectile(Projectile::Hovl(3)),
            ..CardConfig::default()
        },
    }
}

pub fn simple_hammer() -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleHammer,
        sets: vec![SetName::Basics],
        cost: cost(3),
        image: rexard_images::weapon(RexardWeaponType::Hammers, "hmmr_f_b_01"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: CardStats {
                base_attack: Some(1),
                attack_boost: Some(AttackBoost { cost: 1, bonus: 1 }),
                ..CardStats::default()
            },
            lineage: Some(Lineage::Infernal),
            special_effects: projectile(Projectile::Hovl(4)),
            ..CardConfig::default()
        },
    }
}

pub fn simple_spear() -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleSpear,
        sets: vec![SetName::Basics],
        cost: cost(4),
        image: rexard_images::weapon(RexardWeaponType::Polearms, "sp_b_08"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: CardStats {
                base_attack: Some(0),
                attack_boost: Some(AttackBoost { cost: 3, bonus: 5 }),
                ..CardStats::default()
            },
            lineage: Some(Lineage::Infernal),
            special_effects: projectile(Projectile::Hovl(4)),
            ..CardConfig::default()
        },
    }
}

pub fn ethereal_blade() -> CardDefinition {
    CardDefinition {
        name: CardName::EtherealBlade,
        sets: vec![SetName::Basics],
        cost: cost(1),
        image: rexard_images::weapon(RexardWeaponType::Swords, "sv_b_01"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::encounter_boost(),
            Ability {
                text: text!["When you use this weapon, sacrifice it at the end of the raid."],
                ability_type: AbilityType::Standard,
                delegates: vec![
                    on_weapon_used(
                        |_g, s, used_weapon| used_weapon.weapon_id == s.card_id(),
                        |g, s, used_weapon| save_raid_id(g, s, &used_weapon.raid_id),
                    ),
                    on_raid_ended(matching_raid, |g, s, _| {
                        mutations::sacrifice_card(g, s.card_id())?;
                        alert(g, s);
                        Ok(())
                    }),
                ],
            },
        ],
        config: CardConfig {
            stats: attack(1, AttackBoost { cost: 1, bonus: 1 }),
            lineage: Some(Lineage::Prismatic),
            special_effects: projectile(Projectile::Hovl(3)),
            ..CardConfig::default()
        },
    }
}
