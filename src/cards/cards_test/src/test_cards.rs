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

//! Test cards

use card_helpers::costs::actions;
use card_helpers::this::on_activated;
use card_helpers::{abilities, text, *};
use game_data::card_definition::{
    Ability, AbilityType, AttackBoost, CardConfig, CardConfigBuilder, CardDefinition, Resonance,
    SchemePoints,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::delegate_data::{Delegate, QueryDelegate, RaidOutcome};
use game_data::primitives::{CardSubtype, CardType, Rarity, School, Side, Sprite};
use game_data::special_effects::{Projectile, ProjectileData, TimedEffect};
use rules::mutations::OnZeroStored;
use rules::{curses, deal_damage, mutations};

pub fn test_overlord_spell(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestOverlordSpell,
        cost: cost(test_constants::SPELL_COST),
        card_type: CardType::OverlordSpell,
        subtypes: vec![],
        sets: vec![CardSetName::Test],
        image: Sprite::new("Enixion/Fantasy Art Pack 2/Resized/3.png"),
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::None,
        abilities: vec![],
        config: CardConfig::default(),
    }
}

pub fn test_champion_spell(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestChampionSpell,
        cost: cost(test_constants::SPELL_COST),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        sets: vec![CardSetName::Test],
        image: Sprite::new("Enixion/Fantasy Art Pack 2/Resized/2.png"),
        side: Side::Champion,
        school: School::Primal,
        rarity: Rarity::None,
        abilities: vec![],
        config: CardConfig::default(),
    }
}

pub fn test_scheme_310(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestScheme3_10,
        cost: scheme_cost(),
        card_type: CardType::Scheme,
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { level_requirement: 3, points: 10 })
            .build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_scheme_420(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestScheme4_20,
        cost: scheme_cost(),
        card_type: CardType::Scheme,
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { level_requirement: 4, points: 20 })
            .build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_project_2_cost_3_raze(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestProject2Cost3Raze,
        cost: cost(2),
        card_type: CardType::Project,
        config: CardConfigBuilder::new().raze_cost(3).build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_minion_end_raid(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionEndRaid,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![abilities::combat_end_raid()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_minion_shield_1(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionShield1Infernal,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![abilities::combat_end_raid()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .shield(1)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_minion_shield_2_abyssal(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionShield2Abyssal,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![abilities::combat_end_raid()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .shield(2)
            .resonance(Resonance::astral())
            .build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_minion_deal_damage(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionDealDamage,
        cost: cost(1),
        abilities: vec![abilities::combat_deal_damage::<1>()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_minion_infernal(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestInfernalMinion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(Resonance::infernal())
            .build(),
        ..test_minion_end_raid(metadata)
    }
}

pub fn test_minion_astral(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestAstralMinion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(Resonance::astral())
            .build(),
        ..test_minion_end_raid(metadata)
    }
}

pub fn test_minion_astral_1_shield(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestAstralMinion1Shield,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .shield(1)
            .resonance(Resonance::astral())
            .build(),
        ..test_minion_end_raid(metadata)
    }
}

pub fn test_minion_mortal(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMortalMinion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(Resonance::mortal())
            .build(),
        ..test_minion_end_raid(metadata)
    }
}

pub fn test_mortal_minion_2_health(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMortalMinion2Health,
        config: CardConfigBuilder::new().health(2).resonance(Resonance::mortal()).build(),
        ..test_minion_end_raid(metadata)
    }
}

pub fn test_weapon_2_attack(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon2Attack,
        subtypes: vec![CardSubtype::Weapon],
        cost: cost(test_constants::WEAPON_COST),
        card_type: CardType::Artifact,
        config: CardConfigBuilder::new()
            .base_attack(2)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_champion_spell(metadata)
    }
}

pub fn test_weapon_2_attack_12_boost(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon2Attack12Boost,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(2)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(2))
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_weapon_2_attack(metadata)
    }
}

pub fn test_weapon_3_attack_12_boost(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon3Attack12Boost3Cost,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(2))
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_weapon_2_attack(metadata)
    }
}

pub fn test_weapon_4_attack_12_boost(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon4Attack12Boost,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(4)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(2))
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_weapon_2_attack(metadata)
    }
}

pub fn test_weapon_abyssal(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponAbyssal,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(2))
            .resonance(Resonance::astral())
            .build(),
        ..test_weapon_2_attack(metadata)
    }
}

pub fn test_weapon_infernal(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponInfernal,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(2))
            .resonance(Resonance::infernal())
            .build(),
        ..test_weapon_2_attack(metadata)
    }
}

pub fn test_weapon_mortal(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponMortal,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(2))
            .resonance(Resonance::mortal())
            .build(),
        ..test_weapon_2_attack(metadata)
    }
}

pub fn test_weapon_5_attack(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon5AttackInfernal,
        subtypes: vec![CardSubtype::Weapon],
        config: CardConfigBuilder::new()
            .base_attack(5)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_weapon_2_attack(metadata)
    }
}

pub fn activated_ability_take_mana(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestActivatedAbilityTakeMana,
        cost: cost(test_constants::ARTIFACT_COST),
        card_type: CardType::Evocation,
        abilities: vec![
            abilities::store_mana_on_play::<{ test_constants::MANA_STORED }>(),
            abilities::activated_take_mana::<{ test_constants::MANA_TAKEN }>(actions(1)),
        ],
        config: CardConfig::default(),
        ..test_champion_spell(metadata)
    }
}

pub fn triggered_ability_take_mana(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestProjectTriggeredAbilityTakeManaAtDusk,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound],
        abilities: vec![
            projects::store_mana_on_unveil::<{ test_constants::MANA_STORED }>(),
            Ability {
                ability_type: AbilityType::Standard,
                text: trigger_text(Dusk, text![TakeMana(test_constants::MANA_TAKEN)]),
                delegates: vec![in_play::at_dusk(|g, s, _| {
                    mutations::take_stored_mana(
                        g,
                        s.card_id(),
                        test_constants::MANA_TAKEN,
                        OnZeroStored::Sacrifice,
                    )?;
                    Ok(())
                })],
            },
        ],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn duskbound_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestDuskboundProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn roombound_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestRoomboundProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Roombound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn summonbound_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSummonboundProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Summonbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn nightbound_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestNightboundProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Nightbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn dusk_and_nightbound_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestDuskAndNightboundProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound, CardSubtype::Nightbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn trap_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestTrapProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Trap],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_0_cost_champion_spell(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Test0CostChampionSpell,
        cost: cost(0),
        ..test_champion_spell(metadata)
    }
}

pub fn test_1_cost_champion_spell(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Test1CostChampionSpell,
        cost: cost(1),
        ..test_champion_spell(metadata)
    }
}

pub fn deal_damage_end_raid(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionDealDamageEndRaid,
        cost: cost(3),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::combat_deal_damage::<1>(), abilities::combat_end_raid()],
        config: CardConfigBuilder::new()
            .health(5)
            .shield(1)
            .resonance(Resonance::infernal())
            .build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_attack_weapon(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestAttackWeapon,
        cost: cost(3),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(2))
            .resonance(Resonance::infernal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(8))
                    .additional_hit(TimedEffect::SwordSlashes(1)),
            )
            .build(),
        ..test_champion_spell(metadata)
    }
}

pub fn test_sacrifice_draw_card_artifact(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSacrificeDrawCardArtifact,
        cost: cost(test_constants::ARTIFACT_COST),
        card_type: CardType::Artifact,
        abilities: vec![Ability {
            ability_type: abilities::sacrifice_this(),
            text: text!["Draw a card"],
            delegates: vec![on_activated(|g, s, _| {
                mutations::draw_cards(g, s.side(), 1)?;
                Ok(())
            })],
        }],
        config: CardConfig::default(),
        ..test_champion_spell(metadata)
    }
}

pub fn test_sacrifice_end_raid_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestProjectSacrificeToEndRaid,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Roombound],
        abilities: vec![Ability {
            ability_type: abilities::sacrifice_this(),
            text: text!["End the raid"],
            delegates: vec![on_activated(|g, _, _| mutations::end_raid(g, RaidOutcome::Failure))],
        }],
        config: CardConfig::default(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_weapon_reduce_cost_on_raid(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponReduceCostOnSuccessfulRaid,
        cost: cost(5),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Law,
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
        config: CardConfigBuilder::new()
            .base_attack(2)
            .attack_boost(AttackBoost::new().mana_cost(2).bonus(3))
            .resonance(Resonance::infernal())
            .build(),
        ..test_overlord_spell(metadata)
    }
}

pub fn test_spell_give_curse(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSpellGiveCurse,
        cost: cost(0),
        card_type: CardType::OverlordSpell,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Give the Champion a curse"],
            this::on_played(|g, s, _| curses::give_curses(g, s, 1)),
        )],
        ..test_overlord_spell(metadata)
    }
}

pub fn test_evocation(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestEvocation,
        cost: cost(test_constants::EVOCATION_COST),
        card_type: CardType::Evocation,
        sets: vec![CardSetName::Test],
        abilities: vec![],
        ..test_champion_spell(metadata)
    }
}

pub fn test_spell_deal_1_damage(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSpellDeal1Damage,
        cost: cost(0),
        card_type: CardType::OverlordSpell,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Deal 1 damage"],
            this::on_played(|g, s, _| deal_damage::apply(g, s, 1)),
        )],
        ..test_overlord_spell(metadata)
    }
}

pub fn test_spell_deal_5_damage(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSpellDeal5Damage,
        cost: cost(0),
        card_type: CardType::OverlordSpell,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Deal 5 damage"],
            this::on_played(|g, s, _| deal_damage::apply(g, s, 5)),
        )],
        ..test_overlord_spell(metadata)
    }
}
