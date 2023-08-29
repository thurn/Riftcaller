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

use card_helpers::{abilities, text, *};
use game_data::card_definition::{
    Ability, AbilityType, AttackBoost, CardConfig, CardConfigBuilder, CardDefinition, SchemePoints,
    SpecialEffects,
};
use game_data::card_name::CardName;
use game_data::card_set_name::CardSetName;
use game_data::delegates::RaidOutcome;
use game_data::primitives::{CardSubtype, CardType, Rarity, Resonance, School, Side, Sprite};
use game_data::special_effects::{Projectile, TimedEffect};
use rules::mutations;
use rules::mutations::OnZeroStored;

pub fn test_overlord_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::TestOverlordSpell,
        cost: cost(1),
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

pub fn test_champion_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::TestChampionSpell,
        cost: cost(1),
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

pub fn test_scheme_315() -> CardDefinition {
    CardDefinition {
        name: CardName::TestScheme3_15,
        cost: scheme_cost(),
        card_type: CardType::Scheme,
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { level_requirement: 3, points: 15 })
            .build(),
        ..test_overlord_spell()
    }
}

pub fn test_project_2_cost_3_raze() -> CardDefinition {
    CardDefinition {
        name: CardName::TestProject2Cost3Raze,
        cost: cost(2),
        card_type: CardType::Project,
        config: CardConfigBuilder::new().raze_cost(3).build(),
        ..test_overlord_spell()
    }
}

pub fn test_minion_end_raid() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionEndRaid,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![abilities::combat_end_raid()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_overlord_spell()
    }
}

pub fn test_minion_shield_1() -> CardDefinition {
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
        ..test_overlord_spell()
    }
}

pub fn test_minion_shield_2_abyssal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionShield2Abyssal,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![abilities::combat_end_raid()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .shield(2)
            .resonance(Resonance::Abyssal)
            .build(),
        ..test_overlord_spell()
    }
}

pub fn test_minion_deal_damage() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionDealDamage,
        cost: cost(1),
        abilities: vec![abilities::combat_deal_damage::<1>()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_overlord_spell()
    }
}

pub fn test_minion_infernal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestInfernalMinion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(Resonance::Infernal)
            .build(),
        ..test_minion_end_raid()
    }
}

pub fn test_minion_abyssal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestAbyssalMinion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(Resonance::Abyssal)
            .build(),
        ..test_minion_end_raid()
    }
}

pub fn test_minion_mortal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMortalMinion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(Resonance::Mortal)
            .build(),
        ..test_minion_end_raid()
    }
}

pub fn test_weapon_2_attack() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon2Attack,
        subtypes: vec![CardSubtype::Weapon],
        cost: cost(test_constants::WEAPON_COST),
        card_type: CardType::Artifact,
        config: CardConfigBuilder::new()
            .base_attack(2)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_champion_spell()
    }
}

pub fn test_weapon_2_attack_12_boost() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon2Attack12Boost,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(2)
            .attack_boost(AttackBoost { cost: 1, bonus: 2 })
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_3_attack_12_boost() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon3Attack12Boost3Cost,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost { cost: 1, bonus: 2 })
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_4_attack_12_boost() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon4Attack12Boost,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(4)
            .attack_boost(AttackBoost { cost: 1, bonus: 2 })
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_abyssal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponAbyssal,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost { cost: 1, bonus: 2 })
            .resonance(Resonance::Abyssal)
            .build(),
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_infernal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponInfernal,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost { cost: 1, bonus: 2 })
            .resonance(Resonance::Infernal)
            .build(),
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_mortal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponMortal,
        subtypes: vec![CardSubtype::Weapon],
        abilities: vec![abilities::encounter_boost()],
        config: CardConfigBuilder::new()
            .base_attack(3)
            .attack_boost(AttackBoost { cost: 1, bonus: 2 })
            .resonance(Resonance::Mortal)
            .build(),
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_5_attack() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon5Attack,
        subtypes: vec![CardSubtype::Weapon],
        config: CardConfigBuilder::new()
            .base_attack(5)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_weapon_2_attack()
    }
}

pub fn activated_ability_take_mana() -> CardDefinition {
    CardDefinition {
        name: CardName::TestActivatedAbilityTakeMana,
        cost: cost(test_constants::ARTIFACT_COST),
        card_type: CardType::Evocation,
        abilities: vec![
            abilities::store_mana_on_play::<{ test_constants::MANA_STORED }>(),
            abilities::activated_take_mana::<{ test_constants::MANA_TAKEN }>(actions(1)),
        ],
        config: CardConfig::default(),
        ..test_champion_spell()
    }
}

pub fn triggered_ability_take_mana() -> CardDefinition {
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
        ..test_overlord_spell()
    }
}

pub fn duskbound_project() -> CardDefinition {
    CardDefinition {
        name: CardName::TestDuskboundProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell()
    }
}

pub fn roombound_project() -> CardDefinition {
    CardDefinition {
        name: CardName::TestRoomboundProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Roombound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell()
    }
}

pub fn summonbound_project() -> CardDefinition {
    CardDefinition {
        name: CardName::TestSummonboundProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Summonbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell()
    }
}

pub fn nightbound_project() -> CardDefinition {
    CardDefinition {
        name: CardName::TestNightboundProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Nightbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell()
    }
}

pub fn dusk_and_nightbound_project() -> CardDefinition {
    CardDefinition {
        name: CardName::TestDuskAndNightboundProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound, CardSubtype::Nightbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell()
    }
}

pub fn trap_project() -> CardDefinition {
    CardDefinition {
        name: CardName::TestTrapProject,
        cost: cost(test_constants::UNVEIL_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Trap],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_overlord_spell()
    }
}

pub fn test_0_cost_champion_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::Test0CostChampionSpell,
        cost: cost(0),
        ..test_champion_spell()
    }
}

pub fn test_1_cost_champion_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::Test1CostChampionSpell,
        cost: cost(1),
        ..test_champion_spell()
    }
}

pub fn deal_damage_end_raid() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionDealDamageEndRaid,
        cost: cost(3),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::combat_deal_damage::<1>(), abilities::combat_end_raid()],
        config: CardConfigBuilder::new().health(5).shield(1).resonance(Resonance::Infernal).build(),
        ..test_overlord_spell()
    }
}

pub fn test_attack_weapon() -> CardDefinition {
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
            .attack_boost(AttackBoost { cost: 1, bonus: 2 })
            .resonance(Resonance::Infernal)
            .special_effects(SpecialEffects {
                projectile: Some(Projectile::Hovl(8)),
                additional_hit: Some(TimedEffect::HovlSwordSlash(1)),
            })
            .build(),
        ..test_champion_spell()
    }
}

pub fn test_sacrifice_draw_card_artifact() -> CardDefinition {
    CardDefinition {
        name: CardName::TestSacrificeDrawCardArtifact,
        cost: cost(test_constants::ARTIFACT_COST),
        card_type: CardType::Evocation,
        abilities: vec![Ability {
            ability_type: abilities::sacrifice_this(),
            text: text!["Draw a card"],
            delegates: vec![on_activated(|g, s, _| {
                mutations::draw_cards(g, s.side(), 1)?;
                Ok(())
            })],
        }],
        config: CardConfig::default(),
        ..test_champion_spell()
    }
}

pub fn test_sacrifice_end_raid_project() -> CardDefinition {
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
        ..test_overlord_spell()
    }
}
