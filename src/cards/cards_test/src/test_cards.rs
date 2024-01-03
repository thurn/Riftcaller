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

//! Test cards

use card_definition_data::ability_data::{Ability, AbilityType};
use card_definition_data::card_definition::CardDefinition;
use card_helpers::costs::{actions, scheme};
use card_helpers::text_helpers::named_trigger;
use card_helpers::this::on_activated;
use card_helpers::{abilities, combat_abilities, *};
use core_data::game_primitives::{
    CardSubtype, CardType, InitiatedBy, Rarity, School, Side, Sprite,
};
use game_data::card_configuration::{
    AttackBoost, CardConfig, CardConfigBuilder, Resonance, SchemePoints,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::{CardIdsExt, CardPosition};
use game_data::delegate_data::{GameDelegate, QueryDelegate, RaidOutcome};
use game_data::special_effects::{Projectile, ProjectileData, TimedEffect};
use rules::mutations::{OnZeroStored, SummonMinion};
use rules::{curses, damage, destroy, draw_cards, end_raid, mutations, CardDefinitionExt};

pub fn test_ritual(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestRitual,
        cost: cost(test_constants::SPELL_COST),
        card_type: CardType::Ritual,
        subtypes: vec![],
        sets: vec![CardSetName::Test],
        image: Sprite::new("Enixion/Fantasy Art Pack 2/Resized/3.png"),
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::None,
        abilities: vec![],
        config: CardConfig::default(),
    }
}

pub fn test_spell(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSpell,
        cost: cost(test_constants::SPELL_COST),
        card_type: CardType::Spell,
        subtypes: vec![],
        sets: vec![CardSetName::Test],
        image: Sprite::new("Enixion/Fantasy Art Pack 2/Resized/2.png"),
        side: Side::Riftcaller,
        school: School::Primal,
        rarity: Rarity::None,
        abilities: vec![],
        config: CardConfig::default(),
    }
}

pub fn test_singleton_set_spell(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSingletonSetSpell,
        sets: vec![CardSetName::TestSingletonSpellSet],
        rarity: Rarity::Common,
        ..test_spell(metadata)
    }
}

pub fn test_scheme_310(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestScheme3_10,
        cost: scheme(),
        card_type: CardType::Scheme,
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 3, points: 10 })
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_scheme_420(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestScheme4_20,
        cost: scheme(),
        card_type: CardType::Scheme,
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 4, points: 20 })
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_scheme_110(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestScheme1_10,
        cost: scheme(),
        card_type: CardType::Scheme,
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 1, points: 10 })
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_project_2_cost_3_raze(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestProject2Cost3Raze,
        cost: cost(2),
        card_type: CardType::Project,
        config: CardConfigBuilder::new().raze_cost(3).build(),
        ..test_ritual(metadata)
    }
}

pub fn test_minion_end_raid(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionEndRaid,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![combat_abilities::end_raid()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_minion_shield_1(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionShield1Infernal,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![combat_abilities::end_raid()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .shield(1)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_minion_shield_2_astral(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionShield2Astral,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![combat_abilities::end_raid()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .shield(2)
            .resonance(Resonance::astral())
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_minion_shield_3_mortal(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionShield3Mortal,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![combat_abilities::end_raid()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .shield(3)
            .resonance(Resonance::mortal())
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_minion_deal_damage(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionDealDamage,
        cost: cost(1),
        abilities: vec![combat_abilities::deal_damage::<1>()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_minion_lose_mana(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionLoseMana,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![combat_abilities::lose_mana::<1>()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_minion_lose_action_points(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionLoseActionPoints,
        cost: cost(test_constants::MINION_COST),
        abilities: vec![combat_abilities::lose_action_points::<1>()],
        card_type: CardType::Minion,
        config: CardConfigBuilder::new()
            .health(test_constants::MINION_HEALTH)
            .resonance(test_constants::TEST_RESONANCE)
            .build(),
        ..test_ritual(metadata)
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
        ..test_spell(metadata)
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
        name: CardName::TestAstralWeapon,
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
        name: CardName::TestInfernalWeapon,
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
        name: CardName::TestMortalWeapon,
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
            abilities::store_mana_on_play_discard_on_empty::<{ test_constants::MANA_STORED }>(),
            abilities::activated_take_mana::<{ test_constants::MANA_TAKEN }>(actions(1)),
        ],
        config: CardConfig::default(),
        ..test_spell(metadata)
    }
}

pub fn triggered_ability_take_mana(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestProjectTriggeredAbilityTakeManaAtDusk,
        cost: cost(test_constants::SUMMON_PROJECT_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound],
        abilities: vec![
            projects::store_mana_on_summon::<{ test_constants::MANA_STORED }>(),
            Ability {
                ability_type: AbilityType::Standard,
                text: named_trigger(Dusk, text![TakeMana(test_constants::MANA_TAKEN)]),
                delegates: abilities::game(vec![in_play::at_dusk(|g, s, _| {
                    mutations::take_stored_mana(
                        g,
                        s.card_id(),
                        test_constants::MANA_TAKEN,
                        OnZeroStored::Sacrifice,
                    )?;
                    Ok(())
                })]),
            },
        ],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_ritual(metadata)
    }
}

pub fn duskbound_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestDuskboundProject,
        cost: cost(test_constants::SUMMON_PROJECT_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_ritual(metadata)
    }
}

pub fn roombound_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestRoomboundProject,
        cost: cost(test_constants::SUMMON_PROJECT_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Roombound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_ritual(metadata)
    }
}

pub fn summonbound_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSummonboundProject,
        cost: cost(test_constants::SUMMON_PROJECT_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Summonbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_ritual(metadata)
    }
}

pub fn nightbound_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestNightboundProject,
        cost: cost(test_constants::SUMMON_PROJECT_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Nightbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_ritual(metadata)
    }
}

pub fn dusk_and_nightbound_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestDuskAndNightboundProject,
        cost: cost(test_constants::SUMMON_PROJECT_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Duskbound, CardSubtype::Nightbound],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_ritual(metadata)
    }
}

pub fn trap_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestTrapProject,
        cost: cost(test_constants::SUMMON_PROJECT_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Trap],
        abilities: vec![],
        config: CardConfigBuilder::new().raze_cost(test_constants::RAZE_COST).build(),
        ..test_ritual(metadata)
    }
}

pub fn test_0_cost_spell(metadata: CardMetadata) -> CardDefinition {
    CardDefinition { name: CardName::Test0CostSpell, cost: cost(0), ..test_spell(metadata) }
}

pub fn test_0_cost_ritual(metadata: CardMetadata) -> CardDefinition {
    CardDefinition { name: CardName::Test0CostRitual, cost: cost(0), ..test_ritual(metadata) }
}

pub fn test_1_cost_riftcaller_spell(metadata: CardMetadata) -> CardDefinition {
    CardDefinition { name: CardName::Test1CostSpell, cost: cost(1), ..test_spell(metadata) }
}

pub fn deal_damage_end_raid(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionDealDamageEndRaid,
        cost: cost(3),
        card_type: CardType::Minion,
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![combat_abilities::deal_damage::<1>(), combat_abilities::end_raid()],
        config: CardConfigBuilder::new()
            .health(5)
            .shield(1)
            .resonance(Resonance::infernal())
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_attack_weapon(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestAttackWeapon,
        cost: cost(3),
        card_type: CardType::Artifact,
        side: Side::Riftcaller,
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
        ..test_spell(metadata)
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
            delegates: abilities::game(vec![on_activated(|g, s, _| {
                draw_cards::run(g, s.side(), 1, s.initiated_by())?;
                Ok(())
            })]),
        }],
        config: CardConfig::default(),
        ..test_spell(metadata)
    }
}

pub fn test_sacrifice_end_raid_project(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestProjectSacrificeToEndRaid,
        cost: cost(test_constants::SUMMON_PROJECT_COST),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Roombound],
        abilities: vec![Ability {
            ability_type: abilities::sacrifice_this(),
            text: text!["End the raid"],
            delegates: abilities::game(vec![on_activated(|g, s, _| {
                end_raid::run(g, InitiatedBy::Ability(s.ability_id()), RaidOutcome::Failure)
            })]),
        }],
        config: CardConfig::default(),
        ..test_ritual(metadata)
    }
}

pub fn test_weapon_reduce_cost_on_raid(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponReduceCostOnSuccessfulRaid,
        cost: cost(5),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Riftcaller,
        school: School::Law,
        abilities: vec![
            Ability {
                ability_type: AbilityType::Standard,
                text: text![
                    "When you access a room, this weapon costs",
                    ManaMinus(2),
                    "to play this turn"
                ],
                delegates: abilities::game(vec![GameDelegate::ManaCost(QueryDelegate {
                    requirement: this_card,
                    transformation: |g, _, _, value| {
                        if history::rooms_accessed_this_turn(g).count() > 0 {
                            value.map(|v| v.saturating_sub(2))
                        } else {
                            value
                        }
                    },
                })]),
            },
            abilities::encounter_boost(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(2)
            .attack_boost(AttackBoost::new().mana_cost(2).bonus(3))
            .resonance(Resonance::infernal())
            .build(),
        ..test_ritual(metadata)
    }
}

pub fn test_spell_give_curse(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestRitualGiveCurse,
        cost: cost(0),
        card_type: CardType::Ritual,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Give the Riftcaller a curse"],
            this::on_played(|g, s, _| curses::give_curses(g, s, 1)),
        )],
        ..test_ritual(metadata)
    }
}

pub fn test_evocation(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestEvocation,
        cost: cost(test_constants::EVOCATION_COST),
        card_type: CardType::Evocation,
        sets: vec![CardSetName::Test],
        abilities: vec![],
        ..test_spell(metadata)
    }
}

pub fn test_ally(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestAlly,
        cost: cost(test_constants::ALLY_COST),
        card_type: CardType::Ally,
        sets: vec![CardSetName::Test],
        abilities: vec![],
        ..test_spell(metadata)
    }
}

pub fn test_spell_deal_1_damage(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSpellDeal1Damage,
        cost: cost(0),
        card_type: CardType::Ritual,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Deal 1 damage"],
            this::on_played(|g, s, _| damage::deal(g, s, 1)),
        )],
        ..test_ritual(metadata)
    }
}

pub fn test_spell_deal_5_damage(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSpellDeal5Damage,
        cost: cost(0),
        card_type: CardType::Ritual,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Deal 5 damage"],
            this::on_played(|g, s, _| damage::deal(g, s, 5)),
        )],
        ..test_ritual(metadata)
    }
}

pub fn test_ritual_return_discard_to_hand(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestRitualReturnDiscardToHand,
        cost: cost(0),
        card_type: CardType::Ritual,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Return all cards from the crypt to the sanctum"],
            this::on_played(|g, s, _| {
                mutations::move_cards(
                    g,
                    &g.discard_pile(s.side()).card_ids(),
                    CardPosition::Hand(s.side()),
                )
            }),
        )],
        ..test_ritual(metadata)
    }
}

pub fn test_ritual_return_all_occupants_to_hand(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestRitualReturnAllOccupantsToHand,
        cost: cost(0),
        card_type: CardType::Ritual,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Return all cards occupying rooms to the sanctum"],
            this::on_played(|g, s, _| {
                mutations::move_cards(
                    g,
                    &g.occupants_in_all_rooms().card_ids(),
                    CardPosition::Hand(s.side()),
                )
            }),
        )],
        ..test_ritual(metadata)
    }
}

pub fn test_ritual_summon_all_minions(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestRitualSummonAllMinions,
        cost: cost(0),
        card_type: CardType::Ritual,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Summon all minions, ignoring all costs"],
            this::on_played(|g, s, _| {
                let minions = g
                    .all_permanents(Side::Covenant)
                    .filter(|c| c.definition().is_minion())
                    .card_ids();
                for minion in minions {
                    mutations::summon_minion(
                        g,
                        minion,
                        s.initiated_by(),
                        SummonMinion::IgnoreCosts,
                    )?;
                }
                Ok(())
            }),
        )],
        ..test_ritual(metadata)
    }
}

pub fn test_spell_return_all_your_permanents_to_hand(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestSpellReturnAllYourPermanentsToHand,
        cost: cost(0),
        card_type: CardType::Spell,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Return all Riftcaller permanents to hand"],
            this::on_played(|g, s, _| {
                mutations::move_cards(
                    g,
                    &g.all_permanents(Side::Riftcaller).card_ids(),
                    CardPosition::Hand(s.side()),
                )
            }),
        )],
        ..test_spell(metadata)
    }
}

pub fn test_ritual_destroy_all_enemy_permanents(metadata: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestRitualDestroyAllEnemyPermanents,
        cost: cost(0),
        card_type: CardType::Ritual,
        sets: vec![CardSetName::Test],
        abilities: vec![Ability::new_with_delegate(
            text!["Destroy all Riftcaller permanents"],
            this::on_played(|g, s, _| {
                let card_ids = g.all_permanents(Side::Riftcaller).card_ids();
                destroy::run(g, card_ids, s.initiated_by())?;
                Ok(())
            }),
        )],
        ..test_ritual(metadata)
    }
}

pub fn test_ally_access_additional_sanctum_card(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestAllyAccessAdditionalSanctumCard,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(5),
        card_type: CardType::Ally,
        subtypes: vec![],
        abilities: vec![Ability::new(text![
            "When you raid the",
            Sanctum,
            ", access an additional card"
        ])
        .delegate(in_play::on_query_sanctum_access_count(|_, _, _, current| current + 1))],
        ..test_spell(meta)
    }
}

pub fn test_ally_access_additional_vault_card(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TestAllyAccessAdditionalVaultCard,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(5),
        card_type: CardType::Ally,
        subtypes: vec![],
        abilities: vec![Ability::new(text![
            "When you raid the",
            Vault,
            ", access an additional card"
        ])
        .delegate(in_play::on_query_vault_access_count(|_, _, _, current| current + 1))],
        ..test_spell(meta)
    }
}
