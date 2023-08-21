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

//! Card definitions for the Weapon card type

use assets::rexard_images;
use assets::rexard_images::RexardWeaponType;
use card_helpers::{abilities, text, *};
use game_data::card_definition::{
    Ability, AbilityType, AttackBoost, CardConfig, CardDefinition, CardStats, SpecialEffects,
};
use game_data::card_name::CardName;
use game_data::card_set_name::CardSetName;
use game_data::delegates::{Delegate, QueryDelegate};
use game_data::primitives::{CardType, Lineage, Rarity, School, Side};
use game_data::special_effects::{Projectile, TimedEffect};

pub fn marauders_axe() -> CardDefinition {
    CardDefinition {
        name: CardName::MaraudersAxe,
        sets: vec![CardSetName::Amethyst],
        cost: cost(5),
        image: rexard_images::weapon(RexardWeaponType::Axes, "a_n_b_01"),
        card_type: CardType::Weapon,
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
                        if history::raid_accesses_this_turn(g).count() > 0 {
                            value.map(|v| v.saturating_sub(2))
                        } else {
                            value
                        }
                    },
                })],
            },
            abilities::encounter_boost(),
        ],
        config: CardConfig {
            stats: attack(2, AttackBoost { cost: 2, bonus: 3 }),
            lineage: Some(Lineage::Infernal),
            special_effects: SpecialEffects {
                projectile: Some(Projectile::Hovl(8)),
                additional_hit: Some(TimedEffect::HovlSwordSlash(1)),
            },
            ..CardConfig::default()
        },
    }
}

pub fn keen_halberd() -> CardDefinition {
    CardDefinition {
        name: CardName::KeenHalberd,
        sets: vec![CardSetName::Amethyst],
        cost: cost(3),
        image: rexard_images::weapon(RexardWeaponType::Polearms, "sp_b_04"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: CardStats {
                base_attack: Some(3),
                attack_boost: Some(AttackBoost { cost: 2, bonus: 1 }),
                breach: Some(1),
                ..CardStats::default()
            },
            lineage: Some(Lineage::Abyssal),
            special_effects: projectile(Projectile::Hovl(2)),
            ..CardConfig::default()
        },
    }
}

pub fn bow_of_the_alliance() -> CardDefinition {
    CardDefinition {
        name: CardName::BowOfTheAlliance,
        sets: vec![CardSetName::Amethyst],
        cost: cost(3),
        image: rexard_images::weapon(RexardWeaponType::Bows, "b_b_01"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::silent_ability(abilities::encounter_boost()),
            simple_ability(
                encounter_ability_text(
                    text![EncounterBoostCost],
                    text![Plus(1), Attack, "per weapon you control"],
                ),
                Delegate::AttackBoost(QueryDelegate {
                    requirement: this_card,
                    transformation: |g, _s, _, boost| AttackBoost {
                        bonus: g.weapons().count() as u32,
                        ..boost
                    },
                }),
            ),
        ],
        config: CardConfig {
            stats: attack(1, AttackBoost { cost: 1, bonus: 0 }),
            lineage: Some(Lineage::Mortal),
            special_effects: projectile(Projectile::Hovl(4)),
            ..CardConfig::default()
        },
    }
}