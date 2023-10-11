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

use card_helpers::updates::Updates;
use card_helpers::{abilities, costs, history, in_play, text, this};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_definition::{AttackBoost, CardConfigBuilder, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardPosition;
use game_data::primitives::{CardSubtype, CardType, GameObjectId, Rarity, Resonance, School, Side};
use game_data::special_effects::{
    Projectile, ProjectileData, SoundEffect, TimedEffect, TimedEffectData,
};
use game_data::text::TextToken::*;
use rules::mutations;

pub fn pathfinder(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Pathfinder,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(3),
        image: assets::champion_card(meta, "pathfinder"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::standard(
                text![Plus(meta.upgrade(2, 4)), "attack in", OuterRooms],
                this::base_attack(|g, s, _, current| {
                    let Some(raid) = &g.raid else {
                        return current;
                    };

                    current + raid.target.is_outer_room().then_some(s.upgrade(2, 4)).unwrap_or(0)
                }),
            ),
            abilities::encounter_boost(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(1)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(1))
            .resonance(Resonance::Infernal)
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(4))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic2_Projectile01"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagic2_LightImpact01")),
            )
            .build(),
    }
}

pub fn staff_of_the_valiant(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::StaffOfTheValiant,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::champion_card(meta, "staff_of_the_valiant"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Runic],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::standard(
            abilities::encounter_ability_text(
                text![EncounterBoostCost],
                text![EncounterBoostBonus, "for the remainder of this raid"],
            ),
            this::base_attack(|g, s, _, current| {
                let Some(raid_id) = g.raid_id() else {
                    return current;
                };

                let added = history::weapons_used_this_turn(g)
                    .filter_map(|event| {
                        (event.raid_id == raid_id && event.data.weapon_id == s.card_id())
                            .then_some(event.data.attack_boost)
                    })
                    .sum::<u32>();
                current + added
            }),
        )],
        config: CardConfigBuilder::new()
            .base_attack(1)
            .attack_boost(AttackBoost::new().mana_cost(2).bonus(1))
            .resonance(Resonance::Infernal)
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(13))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic2_Projectile02"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagic2_LightImpact02")),
            )
            .build(),
    }
}

pub fn triumph(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Triumph,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(8, 5)),
        image: assets::champion_card(meta, "triumph"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::standard(
                text![
                "The first time this weapon defeats a minion each turn, return that minion to the",
                Sanctum
            ],
                this::on_weapon_used(|g, s, weapon| {
                    if history::weapons_used_this_turn(g).all(|w| w.data.weapon_id != s.card_id()) {
                        Updates::new(g)
                            .timed_effect(
                                GameObjectId::CardId(weapon.data.target_id),
                                TimedEffectData::new(TimedEffect::MagicCircles1(6))
                                    .scale(2.0)
                                    .sound(SoundEffect::LightMagic("RPG3_LightMagic_Cast01"))
                                    .effect_color(design::YELLOW_900),
                            )
                            .ability_alert(s)
                            .apply();

                        mutations::move_card(
                            g,
                            weapon.data.target_id,
                            CardPosition::Hand(Side::Overlord),
                        )?;
                    }
                    Ok(())
                }),
            ),
            abilities::slow(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(0)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(1))
            .resonance(Resonance::Astral)
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(15))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic3_Projectile03"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_Impact01")),
            )
            .build(),
    }
}

pub fn spear_of_conquest(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SpearOfConquest,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::champion_card(meta, "spear_of_conquest"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Charge],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::standard(
            text!["When you access a room, add", PowerCharges(1)],
            in_play::on_raid_access_start(|g, s, _| {
                mutations::add_power_charges(g, s.card_id(), 1)
            }),
        )],
        config: CardConfigBuilder::new()
            .base_attack(1)
            .resonance(Resonance::Mortal)
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(23))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic2_Projectile01"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagic2_LightImpact01")),
            )
            .build(),
    }
}
