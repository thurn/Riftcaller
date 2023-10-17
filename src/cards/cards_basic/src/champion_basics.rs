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
use card_helpers::abilities::standard;
use card_helpers::costs::actions;
use card_helpers::effects::Effects;
use card_helpers::this::on_activated;
use card_helpers::*;
use game_data::card_definition::{
    Ability, AbilityType, AttackBoost, CardConfig, CardConfigBuilder, CardDefinition,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::primitives::{CardSubtype, CardType, Rarity, Resonance, School, Side};
use game_data::special_effects::{Projectile, ProjectileData, TimedEffect};
use rules::mutations::{add_stored_mana, OnZeroStored};
use rules::{mana, mutations};

pub fn arcane_recovery(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ArcaneRecovery,
        sets: vec![CardSetName::Basics],
        cost: cost(5),
        image: rexard_images::spell(1, "SpellBook01_24"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![standard(
            text![Gain, Mana(9)],
            this::on_played(|g, s, _| {
                mana::gain(g, s.side(), 9);
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn eldritch_surge(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EldritchSurge,
        sets: vec![CardSetName::Basics],
        cost: cost(0),
        image: rexard_images::spell(1, "SpellBook01_56"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![standard(
            text![Gain, Mana(3)],
            this::on_played(|g, s, _| {
                mana::gain(g, s.side(), 3);
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn lodestone(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Lodestone,
        sets: vec![CardSetName::Basics],
        cost: cost(1),
        image: rexard_images::get(RexardPack::MagicItems, "orb_04_b"),
        card_type: CardType::Evocation,
        subtypes: vec![],
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

pub fn mana_battery(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ManaBattery,
        sets: vec![CardSetName::Basics],
        cost: cost(0),
        image: rexard_images::get(RexardPack::MagicItems, "artifact_11_b"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![
            standard(
                trigger_text(Dawn, text![TakeMana(1)]),
                in_play::at_dawn(|g, s, _| {
                    let taken =
                        mutations::take_stored_mana(g, s.card_id(), 1, OnZeroStored::Ignore)?;
                    Effects::new().ability_alert_if_nonzero(s, taken).apply(g);
                    Ok(())
                }),
            ),
            Ability {
                ability_type: activate_for_action(),
                text: text![StoreMana(3)],
                delegates: vec![on_activated(|g, s, _| {
                    add_stored_mana(g, s.card_id(), 3);
                    Ok(())
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn contemplate(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Contemplate,
        sets: vec![CardSetName::Basics],
        cost: cost(0),
        image: rexard_images::spell(2, "SpellBook02_01"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![standard(
            text![text![Gain, Mana(2)], text!["Draw a card"]],
            this::on_played(|g, s, _| {
                mana::gain(g, s.side(), 2);
                mutations::draw_cards(g, s.side(), 1)?;
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn ancestral_knowledge(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::AncestralKnowledge,
        sets: vec![CardSetName::Basics],
        cost: cost(1),
        image: rexard_images::spell(3, "SpellBook03_46"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![standard(
            text!["Draw", 3, "cards"],
            this::on_played(|g, s, _| {
                mutations::draw_cards(g, s.side(), 3)?;
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn simple_blade(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleBlade,
        sets: vec![CardSetName::Basics],
        cost: cost(4),
        image: rexard_images::weapon(RexardWeaponType::Swords, "swnb_01"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(2)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(1))
            .resonance(Resonance::Mortal)
            .combat_projectile(ProjectileData::new(Projectile::Projectiles1(2)))
            .build(),
    }
}

pub fn simple_axe(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleAxe,
        sets: vec![CardSetName::Basics],
        cost: cost(4),
        image: rexard_images::weapon(RexardWeaponType::Axes, "a_n_b_01"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost::new().mana_cost(3).bonus(1))
            .resonance(Resonance::Mortal)
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(8))
                    .additional_hit(TimedEffect::SwordSlashes(1)),
            )
            .build(),
    }
}

pub fn simple_bow(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleBow,
        sets: vec![CardSetName::Basics],
        cost: cost(0),
        image: rexard_images::weapon(RexardWeaponType::Bows, "b_b_02"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(1)
            .attack_boost(AttackBoost::new().mana_cost(2).bonus(1))
            .resonance(Resonance::Astral)
            .combat_projectile(ProjectileData::new(Projectile::Projectiles1(3)))
            .build(),
    }
}

pub fn simple_club(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleClub,
        sets: vec![CardSetName::Basics],
        cost: cost(2),
        image: rexard_images::weapon(RexardWeaponType::Clubs, "bl_b_07"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(2)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(1))
            .resonance(Resonance::Astral)
            .combat_projectile(ProjectileData::new(Projectile::Projectiles1(3)))
            .build(),
    }
}

pub fn simple_hammer(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleHammer,
        sets: vec![CardSetName::Basics],
        cost: cost(3),
        image: rexard_images::weapon(RexardWeaponType::Hammers, "hmmr_f_b_01"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(1)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(1))
            .resonance(Resonance::Infernal)
            .combat_projectile(ProjectileData::new(Projectile::Projectiles1(4)))
            .build(),
    }
}

pub fn simple_spear(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SimpleSpear,
        sets: vec![CardSetName::Basics],
        cost: cost(4),
        image: rexard_images::weapon(RexardWeaponType::Polearms, "sp_b_08"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(0)
            .attack_boost(AttackBoost::new().mana_cost(3).bonus(5))
            .resonance(Resonance::Infernal)
            .combat_projectile(ProjectileData::new(Projectile::Projectiles1(4)))
            .build(),
    }
}

pub fn ethereal_blade(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EtherealBlade,
        sets: vec![CardSetName::Basics],
        cost: cost(1),
        image: rexard_images::weapon(RexardWeaponType::Swords, "sv_b_01"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::encounter_boost(),
            Ability {
                ability_type: AbilityType::Standard,
                text: text!["When you use this weapon, sacrifice it at the end of the raid"],
                delegates: vec![on_raid_ended(requirements::weapon_used_this_raid, |g, s, _| {
                    mutations::sacrifice_card(g, s.card_id())?;
                    Effects::new().ability_alert(s).apply(g);
                    Ok(())
                })],
            },
        ],
        config: CardConfigBuilder::new()
            .base_attack(1)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(1))
            .resonance(Resonance::Prismatic)
            .combat_projectile(ProjectileData::new(Projectile::Projectiles1(3)))
            .build(),
    }
}
