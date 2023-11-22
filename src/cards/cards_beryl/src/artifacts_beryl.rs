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

use card_helpers::effects::Effects;
use card_helpers::{
    abilities, costs, delegates, history, in_play, raids, requirements, show_prompt, text, this,
};
use core_data::game_primitives::{
    CardSubtype, CardType, GameObjectId, Rarity, RoomId, School, Side, INNER_ROOMS,
};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_definition::{
    Ability, ActivatedAbility, AttackBoost, CardConfig, CardConfigBuilder, CardDefinition,
    CustomBoostCost, CustomWeaponCost, Resonance,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::{CardCounter, CardPosition};
use game_data::delegate_data::{CardInfoElementKind, CardStatusMarker, Scope};
use game_data::game_actions::{CardTarget, PromptChoice};
use game_data::game_effect::GameEffect;
use game_data::game_state::{GameState, RaidJumpRequest};
use game_data::special_effects::{
    Projectile, ProjectileData, SoundEffect, TimedEffect, TimedEffectData,
};
use game_data::text::TextToken::*;
use game_data::utils;
use rules::mana::ManaPurpose;
use rules::{mana, mutations, queries, CardDefinitionExt};

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
            Ability::new_with_delegate(
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
            .resonance(Resonance::infernal())
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
        abilities: vec![Ability::new_with_delegate(
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
            .resonance(Resonance::infernal())
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
            Ability::new_with_delegate(
                text![
                "The first time this weapon defeats a minion each turn, return that minion to the",
                Sanctum
            ],
                this::on_weapon_used(|g, s, weapon| {
                    if history::weapons_used_this_turn(g).all(|w| w.data.weapon_id != s.card_id()) {
                        Effects::new()
                            .timed_effect(
                                GameObjectId::CardId(weapon.data.target_id),
                                TimedEffectData::new(TimedEffect::MagicCircles1(6))
                                    .scale(2.0)
                                    .sound(SoundEffect::LightMagic("RPG3_LightMagic_Cast01"))
                                    .effect_color(design::YELLOW_900),
                            )
                            .ability_alert(s)
                            .apply(g);

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
            .resonance(Resonance::astral())
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
        abilities: vec![
            Ability::new_with_delegate(
                text!["When you access a room,", AddPowerCharges(1)],
                in_play::on_raid_access_start(|g, s, _| {
                    mutations::add_power_charges(g, s.card_id(), 1)
                }),
            ),
            Ability::new(abilities::encounter_ability_text(
                text![PowerCharges(1)],
                text![EncounterBoostBonus],
            )),
        ],
        config: CardConfigBuilder::new()
            .base_attack(meta.upgrade(1, 2))
            .attack_boost(
                AttackBoost::new().custom_boost_cost(CustomBoostCost::PowerCharges(1)).bonus(1),
            )
            .resonance(Resonance::mortal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(23))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic2_Projectile01"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagic2_LightImpact01")),
            )
            .build(),
    }
}

pub fn blade_of_reckoning(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::BladeOfReckoning,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(2),
        image: assets::champion_card(meta, "blade_of_reckoning"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Charge],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text!["When you access a room,", AddPowerCharges(1)],
                in_play::on_raid_access_start(|g, s, _| {
                    mutations::add_power_charges(g, s.card_id(), 1)
                }),
            ),
            Ability::new(abilities::encounter_ability_text(
                text![PowerCharges(1)],
                text![EncounterBoostBonus],
            )),
        ],
        config: CardConfigBuilder::new()
            .base_attack(meta.upgrade(2, 3))
            .attack_boost(
                AttackBoost::new().custom_boost_cost(CustomBoostCost::PowerCharges(1)).bonus(1),
            )
            .resonance(Resonance::astral())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(23))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic2_Projectile01"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagic2_LightImpact01")),
            )
            .build(),
    }
}

pub fn resolution(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Resolution,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::champion_card(meta, "resolution"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::breach(),
            Ability::new_with_delegate(
                text!["When this weapon defeats a minion, sacrifice it"],
                this::on_weapon_used(|g, s, _| {
                    Effects::new().ability_alert(s).apply(g);
                    mutations::sacrifice_card(g, s.card_id())
                }),
            ),
            abilities::encounter_boost(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(meta.upgrade(2, 4))
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(1))
            .breach(5)
            .resonance(Resonance::mortal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles2(19))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic2_Projectile03"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagic2_LightImpact03")),
            )
            .build(),
    }
}

pub fn starlight_lantern(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::StarlightLantern,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::champion_card(meta, "starlight_lantern"),
        card_type: CardType::Artifact,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text!["When you play an artifact, including this card,", StoreMana(1)],
                in_play::on_card_played(|g, s, played| {
                    if g.card(played.card_id).definition().card_type == CardType::Artifact {
                        Effects::new()
                            .timed_effect(
                                GameObjectId::CardId(s.card_id()),
                                TimedEffectData::new(TimedEffect::MagicCircles1(7))
                                    .scale(2.0)
                                    .sound(SoundEffect::LightMagic("RPG3_LightMagic_Cast02"))
                                    .effect_color(design::YELLOW_900),
                            )
                            .apply(g);
                        mutations::add_stored_mana(g, s.card_id(), 1);
                    }
                    Ok(())
                }),
            ),
            ActivatedAbility::new(costs::sacrifice_for_action(), text!["Take all stored mana"])
                .delegate(this::on_activated(|g, s, _| {
                    mana::gain(
                        g,
                        s.side(),
                        g.card(s.card_id()).last_known_counters(CardCounter::StoredMana),
                    );
                    Ok(())
                }))
                .build(),
        ],
        config: CardConfig::default(),
    }
}

pub fn warriors_sign(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::WarriorsSign,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(3),
        image: assets::champion_card(meta, "warriors_sign"),
        card_type: CardType::Artifact,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![
                "When you start raids on the",
                Vault,
                ",",
                Sanctum,
                ", and",
                Crypt,
                "in a single turn,",
                GainActions(meta.upgrade(1, 2))
            ],
            in_play::on_raid_started(|g, s, raid| {
                if raid.target.is_inner_room()
                    && INNER_ROOMS
                        .into_iter()
                        .filter(|room_id| **room_id != raid.target)
                        .all(|room_id| history::rooms_raided_this_turn(g).any(|r| r == *room_id))
                {
                    Effects::new()
                        .timed_effect(
                            GameObjectId::Character(Side::Champion),
                            TimedEffectData::new(TimedEffect::MagicCircles1(9))
                                .scale(2.0)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagic_Buff03_FULL"))
                                .effect_color(design::YELLOW_900),
                        )
                        .ability_alert(s)
                        .apply(g);

                    mutations::gain_action_points(g, s.side(), s.upgrade(1, 2))?;
                }

                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn chains_of_mortality(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ChainsOfMortality,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(5, 3)),
        image: assets::champion_card(meta, "chains_of_mortality"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text![
                    "The first minion you encounter each turn gains",
                    Mortal,
                    "during that encounter"
                ],
                in_play::on_query_resonance(|g, _, card_id, resonance| {
                    if Some(*card_id) == g.current_raid_defender()
                        && history::minions_encountered_this_turn(g).count() == 1
                    {
                        // Encounters are added to history immediately, so the 'first encounter'
                        // corresponds to an encounter history of length 1.
                        resonance.with_mortal(true)
                    } else {
                        resonance
                    }
                }),
            ),
            abilities::encounter_boost(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(1)
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(1))
            .resonance(Resonance::mortal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(2))
                    .fire_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Projectiles01"))
                    .impact_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Impact01")),
            )
            .build(),
    }
}

pub fn phase_door(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::PhaseDoor,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(5, 3)),
        image: assets::champion_card(meta, "phase_door"),
        card_type: CardType::Artifact,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![ActivatedAbility::new(
            costs::actions(1),
            text![text!["Raid the", Crypt], text!["If successful, access the", Vault, "instead"]],
        )
        .delegate(this::on_activated(|g, s, _| {
            raids::initiate(g, s, CardTarget::Room(RoomId::Crypts))?;
            Ok(())
        }))
        .delegate(delegates::on_raid_access_start(requirements::matching_raid, |g, s, _| {
            Effects::new()
                .timed_effect(
                    GameObjectId::CardId(s.card_id()),
                    TimedEffectData::new(TimedEffect::MagicCircles1(1))
                        .scale(2.0)
                        .sound(SoundEffect::WaterMagic("RPG3_WaterMagic2_Cast"))
                        .effect_color(design::BLUE_900),
                )
                .ability_alert(s)
                .apply(g);

            mutations::apply_raid_jump(g, RaidJumpRequest::ChangeTarget(RoomId::Vault));

            Ok(())
        }))
        .build()],
        config: CardConfig::default(),
    }
}

pub fn skyprism(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Skyprism,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(5),
        image: assets::champion_card(meta, "skyprism"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new(text!["As an additional cost to use this weapon, pay", Actions(1)]),
            abilities::encounter_boost(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(meta.upgrade(0, 2))
            .attack_boost(
                AttackBoost::new()
                    .mana_cost(1)
                    .bonus(1)
                    .custom_weapon_cost(CustomWeaponCost::ActionPoints(1)),
            )
            .resonance(Resonance::prismatic())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(6))
                    .fire_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Projectiles01"))
                    .impact_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Impact01")),
            )
            .build(),
    }
}

pub fn shield_of_the_flames(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ShieldOfTheFlames,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(4),
        image: assets::champion_card(meta, "shield_of_the_flames"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![
            ActivatedAbility::new(costs::sacrifice(), text![Evade, "an", Infernal, "minion"])
                .delegate(this::can_activate(|g, _, _, flag| {
                    flag.add_constraint(utils::is_true(|| {
                        Some(queries::resonance(g, raids::active_encounter_prompt(g)?)?.infernal)
                    }))
                }))
                .delegate(this::on_activated(|g, _, _| {
                    mutations::apply_raid_jump(g, RaidJumpRequest::EvadeCurrentMinion);
                    Ok(())
                }))
                .build(),
            abilities::encounter_boost(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(meta.upgrade(2, 3))
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(1))
            .resonance(Resonance::infernal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(9))
                    .fire_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Projectiles02"))
                    .impact_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Impact01")),
            )
            .build(),
    }
}

pub fn foebane(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Foebane,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(8),
        image: assets::champion_card(meta, "foebane"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![
            abilities::choose_a_minion_in_target_room(),
            Ability::new_with_delegate(
                text!["You may evade that minion by paying its shield cost"],
                in_play::on_minion_approached(|g, s, event| {
                    if Some(event.data) == g.card(s.card_id()).on_play_state().chosen_card() {
                        let shield = queries::shield(g, event.data, None);
                        if mana::get(g, s.side(), ManaPurpose::PayForTriggeredAbility) >= shield {
                            show_prompt::with_choices(
                                g,
                                s,
                                vec![
                                    PromptChoice::new()
                                        .effect(GameEffect::ManaCost(
                                            s.side(),
                                            shield,
                                            s.initiated_by(),
                                        ))
                                        .effect(GameEffect::EvadeCurrentEncounter),
                                    PromptChoice::new().effect(GameEffect::Continue),
                                ],
                            )
                        }
                    }
                    Ok(())
                }),
            )
            .delegate(in_play::on_query_card_status_markers(
                |g, s, card_id, mut markers| {
                    if Some(*card_id) == g.card(s.card_id()).on_play_state().chosen_card() {
                        markers.push(CardStatusMarker {
                            source: s.ability_id(),
                            marker_kind: CardInfoElementKind::NegativeEffect,
                            text: text!["Can evade by paying shield cost"],
                        });
                    }
                    markers
                },
            )),
            abilities::encounter_boost(),
        ],
        config: CardConfigBuilder::new()
            .custom_targeting(requirements::any_room_with_defenders())
            .base_attack(1)
            .attack_boost(AttackBoost::new().mana_cost(2).bonus(meta.upgrade(1, 2)))
            .resonance(Resonance::infernal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(17))
                    .fire_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Projectiles02"))
                    .impact_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Impact01")),
            )
            .note(
                "This card only triggers when approaching a minion, meaning that it bypasses \
                   'on encounter' abilities and cannot be used immediately if played during an \
                   encounter.",
            )
            .build(),
    }
}

pub fn whip_of_disjunction(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::WhipOfDisjunction,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(4),
        image: assets::champion_card(meta, "whip_of_disjunction"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Runic],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![ActivatedAbility::new(
            costs::ability_mana(2),
            text![
                "Abilities of",
                Astral,
                "minions with",
                5,
                "or less health cannot end the raid during the current encounter"
            ],
        )
        .delegate(this::can_activate(|g, _, _, flag| {
            flag.add_constraint(utils::is_true(|| {
                Some(queries::resonance(g, raids::active_encounter_prompt(g)?)?.astral)
            }))
        }))
        .delegate(delegates::can_ability_end_raid(
            requirements::ability_activated_this_encounter,
            |g, _, event, flag| {
                let health = queries::health(g, event.data.card_id);
                let resonance = queries::resonance(g, event.data.card_id);
                flag.add_constraint(health > 5 || resonance.map_or(true, |r| !r.astral))
            },
        ))
        .build()],
        config: CardConfigBuilder::new().resonance(Resonance::astral()).build(),
    }
}

pub fn glimmersong(meta: CardMetadata) -> CardDefinition {
    fn apply_vfx(game: &mut GameState, scope: Scope) {
        Effects::new()
            .timed_effect(
                GameObjectId::CardId(scope.card_id()),
                TimedEffectData::new(TimedEffect::MagicCircles1(2))
                    .scale(2.0)
                    .sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Cast01"))
                    .effect_color(design::BLUE_900),
            )
            .apply(game);
    }

    CardDefinition {
        name: CardName::Glimmersong,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(3),
        image: assets::champion_card(meta, "glimmersong"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Enchanted],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![
            Ability::new_with_delegate(
                text!["When you reveal a card,", AddPowerCharges(1)],
                in_play::on_card_revealed(|g, s, card_id| {
                    if card_id.side != s.side() {
                        g.card_mut(s.card_id()).add_counters(CardCounter::PowerCharges, 1);
                        apply_vfx(g, s);
                    }
                    Ok(())
                }),
            ),
            Ability::new_with_delegate(
                text![
                    "When you access a room without scoring or using a",
                    RazeAbility,
                    ",",
                    AddPowerCharges(1)
                ],
                in_play::on_raid_access_end(|g, s, event| {
                    if history::accessed_cards_razed_this_turn(g)
                        .all(|e| e.room_access_id != event.room_access_id)
                        && history::accessed_cards_scored_this_turn(g)
                            .all(|e| e.room_access_id != event.room_access_id)
                    {
                        g.card_mut(s.card_id()).add_counters(CardCounter::PowerCharges, 1);
                        apply_vfx(g, s);
                    }
                    Ok(())
                }),
            ),
            abilities::plus_1_attack_per_power_charge(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(meta.upgrade(0, 1))
            .resonance(Resonance::prismatic())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(26))
                    .fire_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Projectiles03"))
                    .impact_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Impact03")),
            )
            .build(),
    }
}
