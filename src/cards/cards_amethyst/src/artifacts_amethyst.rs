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

//! Card definitions for the Weapon card type

use core_data::game_primitives::{CardSubtype, CardType, Rarity, School, Side};
use game_data::card_definition::{
    Ability, AbilityType, AttackBoost, CardConfigBuilder, CardDefinition, Resonance,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::delegate_data::{Delegate, QueryDelegate};
use game_data::special_effects::{Projectile, ProjectileData, TimedEffect};

use assets::rexard_images;
use assets::rexard_images::RexardWeaponType;
use card_helpers::{*, abilities};
use card_helpers::abilities::encounter_ability_text;

pub fn marauders_axe(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::MaraudersAxe,
        sets: vec![CardSetName::Amethyst],
        cost: cost(5),
        image: rexard_images::weapon(RexardWeaponType::Axes, "a_n_b_01"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability {
                ability_type: AbilityType::Standard,
                text: text![
                    "When you access a room, this weapon costs",
                    ManaMinus(2),
                    "to play this turn"
                ],
                delegates: vec![Delegate::ManaCost(QueryDelegate {
                    requirement: this_card,
                    transformation: |g, _, _, value| {
                        if history::rooms_accessed_this_turn(g).count() > 0 {
                            value.map(|v| v.saturating_sub(2))
                        } else {
                            value
                        }
                    },
                })],
            },
            abilities::encounter_boost(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(2)
            .attack_boost(AttackBoost::new().mana_cost(2).bonus(3))
            .resonance(Resonance::infernal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(8))
                    .additional_hit(TimedEffect::SwordSlashes(1)),
            )
            .build(),
    }
}

pub fn keen_halberd(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::KeenHalberd,
        sets: vec![CardSetName::Amethyst],
        cost: cost(3),
        image: rexard_images::weapon(RexardWeaponType::Polearms, "sp_b_04"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost::new().mana_cost(2).bonus(1))
            .breach(1)
            .resonance(Resonance::astral())
            .combat_projectile(ProjectileData::new(Projectile::Projectiles1(2)))
            .build(),
    }
}

pub fn bow_of_the_alliance(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::BowOfTheAlliance,
        sets: vec![CardSetName::Amethyst],
        cost: cost(3),
        image: rexard_images::weapon(RexardWeaponType::Bows, "b_b_01"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            encounter_ability_text(
                text![EncounterBoostCost],
                text![Plus(1), Attack, "per weapon you control"],
            ),
            Delegate::AttackBoostBonus(QueryDelegate {
                requirement: this_card,
                transformation: |g, _s, _, current| current + g.artifacts().count() as u32,
            }),
        )],
        config: CardConfigBuilder::new()
            .base_attack(1)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(0))
            .resonance(Resonance::mortal())
            .combat_projectile(ProjectileData::new(Projectile::Projectiles1(4)))
            .build(),
    }
}
