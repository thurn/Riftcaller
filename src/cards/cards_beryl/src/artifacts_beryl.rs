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

use card_definition_data::ability_data::{Ability, ActivatedAbility};
use card_definition_data::card_definition::CardDefinition;
use card_definition_data::cards::CardDefinitionExt;
use card_helpers::{
    abilities, costs, delegates, history, in_play, raids, requirements, show_prompt, text,
    text_helpers, this,
};
use core_data::game_primitives::{
    CardSubtype, CardType, GameObjectId, Rarity, RoomId, School, Side, INNER_ROOMS,
};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_configuration::{
    AttackBoost, CardConfig, CardConfigBuilder, CustomBoostCost, CustomWeaponCost, Resonance,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::{CardCounter, CardPosition};
use game_data::custom_card_state::CustomCardState;
use game_data::delegate_data::{CardInfoElementKind, CardStatusMarker, Scope};
use game_data::game_actions::CardTarget;
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::prompt_data::{PromptChoice, PromptData};
use game_data::raid_data::RaidJumpRequest;
use game_data::special_effects::{
    Projectile, ProjectileData, SoundEffect, TimedEffect, TimedEffectData,
};
use game_data::text::TextToken::*;
use game_data::utils;
use rules::mana::ManaPurpose;
use rules::visual_effects::{ShowAlert, VisualEffects};
use rules::{draw_cards, end_raid, flags, mana, mutations, prompts, queries, visual_effects};
use with_error::WithError;

pub fn pathfinder(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Pathfinder,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(3),
        image: assets::riftcaller_card(meta, "pathfinder"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Riftcaller,
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
        image: assets::riftcaller_card(meta, "staff_of_the_valiant"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Runic],
        side: Side::Riftcaller,
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
        image: assets::riftcaller_card(meta, "triumph"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Riftcaller,
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
                        VisualEffects::new()
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
                            CardPosition::Hand(Side::Covenant),
                        )?;
                    }
                    Ok(())
                }),
            ),
            Ability::new_with_delegate(
                text![
                    abilities::encounter_ability_text(
                        text![EncounterBoostCost],
                        text![EncounterBoostBonus]
                    ),
                    text![SlowAbility]
                ],
                this::is_slow_weapon(|_, _, _, _| true),
            ),
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
        image: assets::riftcaller_card(meta, "spear_of_conquest"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Charge],
        side: Side::Riftcaller,
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
        image: assets::riftcaller_card(meta, "blade_of_reckoning"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Charge],
        side: Side::Riftcaller,
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
        image: assets::riftcaller_card(meta, "resolution"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::breach(),
            Ability::new_with_delegate(
                text!["When this weapon defeats a minion, sacrifice it"],
                this::on_weapon_used(|g, s, _| {
                    VisualEffects::new().ability_alert(s).apply(g);
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
        image: assets::riftcaller_card(meta, "starlight_lantern"),
        card_type: CardType::Artifact,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text!["When you play an artifact, including this card,", StoreMana(1)],
                in_play::on_card_played(|g, s, played| {
                    if g.card(played.card_id).definition().card_type == CardType::Artifact {
                        VisualEffects::new()
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
            ActivatedAbility::new(costs::sacrifice_and_action(), text!["Take all stored mana"])
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
        image: assets::riftcaller_card(meta, "warriors_sign"),
        card_type: CardType::Artifact,
        subtypes: vec![],
        side: Side::Riftcaller,
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
                    VisualEffects::new()
                        .timed_effect(
                            GameObjectId::Character(Side::Riftcaller),
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
        image: assets::riftcaller_card(meta, "chains_of_mortality"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Riftcaller,
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
        image: assets::riftcaller_card(meta, "phase_door"),
        card_type: CardType::Artifact,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![ActivatedAbility::new(
            costs::actions(1),
            text![text!["Raid the", Crypt], text!["If successful, access the", Vault, "instead"]],
        )
        .delegate(this::on_activated(|g, s, _| {
            raids::initiate(g, s, CardTarget::Room(RoomId::Crypt))?;
            Ok(())
        }))
        .delegate(delegates::on_raid_access_start(requirements::matching_raid, |g, s, _| {
            VisualEffects::new()
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
        image: assets::riftcaller_card(meta, "skyprism"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Riftcaller,
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
        image: assets::riftcaller_card(meta, "shield_of_the_flames"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![
            ActivatedAbility::new(costs::sacrifice(), text![Evade, "an", Infernal, "minion"])
                .delegate(this::can_activate(|g, _, _, flag| {
                    flag.add_constraint(utils::is_true(|| {
                        Some(queries::resonance(g, raids::active_encounter_prompt(g)?)?.infernal)
                    }))
                    .add_constraint(flags::can_evade_current_minion(g))
                }))
                .delegate(this::on_activated(|g, _, _| mutations::evade_current_minion(g)))
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
        image: assets::riftcaller_card(meta, "foebane"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![
            abilities::choose_a_minion_in_target_room(),
            Ability::new_with_delegate(
                text!["You may evade that minion by paying its shield cost"],
                in_play::on_minion_approached(|g, s, event| {
                    let card = g.card(s);
                    if card.custom_state.targets_contain(card.last_card_play_id, event.data)
                        && flags::can_evade_current_minion(g)
                    {
                        prompts::push_with_data(
                            g,
                            Side::Riftcaller,
                            s,
                            PromptData::Card(event.data),
                        );
                    }
                    Ok(())
                }),
            )
            .delegate(this::prompt(|g, s, source, _| {
                if let PromptData::Card(card_id) = source.data {
                    let shield = queries::shield(g, card_id, None);
                    if mana::get(g, s.side(), ManaPurpose::PayForTriggeredAbility) >= shield {
                        return show_prompt::with_choices(vec![
                            PromptChoice::new()
                                .effect(GameEffect::ManaCost(s.side(), shield, s.initiated_by()))
                                .effect(GameEffect::EvadeCurrentEncounter),
                            PromptChoice::new().effect(GameEffect::Continue),
                        ]);
                    }
                }

                None
            }))
            .delegate(in_play::on_query_card_status_markers(
                |g, s, card_id, mut markers| {
                    let card = g.card(s);
                    if card.custom_state.targets_contain(card.last_card_play_id, *card_id) {
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
            .custom_targeting(requirements::defended_room())
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
        image: assets::riftcaller_card(meta, "whip_of_disjunction"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Runic],
        side: Side::Riftcaller,
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
        .delegate(this::on_activated(|g, s, _| {
            let encounter_id =
                g.raid()?.minion_encounter_id.with_error(|| "Expected active minion encounter")?;
            g.card_mut(s).custom_state.push(CustomCardState::ActiveForEncounter { encounter_id });
            Ok(())
        }))
        .delegate(delegates::on_ability_will_end_raid(
            requirements::active_this_encounter,
            |g, _, event| {
                let health = queries::health(g, event.data.card_id);
                let resonance = queries::resonance(g, event.data.card_id);
                if !(health > 5 || resonance.map_or(true, |r| !r.astral)) {
                    end_raid::prevent(g)
                }
                Ok(())
            },
        ))
        .build()],
        config: CardConfigBuilder::new().resonance(Resonance::astral()).build(),
    }
}

pub fn glimmersong(meta: CardMetadata) -> CardDefinition {
    fn apply_vfx(game: &mut GameState, scope: Scope) {
        VisualEffects::new()
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
        image: assets::riftcaller_card(meta, "glimmersong"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Enchanted, CardSubtype::Charge],
        side: Side::Riftcaller,
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
                        .all(|e| e.room_access_id() != event.room_access_id)
                        && history::accessed_cards_scored_this_turn(g)
                            .all(|e| e.room_access_id() != event.room_access_id)
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

pub fn spear_of_ultimatum(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SpearOfUltimatum,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::riftcaller_card(meta, "spear_of_ultimatum"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon, CardSubtype::Runic],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![
            ActivatedAbility::new(costs::ability_mana(2), text!["Choose a minion"])
                .target_requirement(requirements::any_room_with_infernal_defenders())
                .delegate(this::on_activated(|g, s, activation| {
                    prompts::push_with_data(
                        g,
                        Side::Riftcaller,
                        s,
                        PromptData::AbilityActivation(*activation),
                    );
                    Ok(())
                }))
                .delegate(this::prompt(|g, s, source, _| {
                    let PromptData::AbilityActivation(activation) = source.data else {
                        return None;
                    };
                    let CardTarget::Room(room_id) = activation.target else {
                        return None;
                    };
                    let play_id = g.card(s).last_card_play_id?;
                    show_prompt::with_choices(
                        g.defenders_unordered(room_id)
                            .filter(|c| c.definition().is_infernal())
                            .map(|card| {
                                PromptChoice::new()
                                    .effect(GameEffect::AppendCustomCardState(
                                        s.card_id(),
                                        CustomCardState::TargetCard {
                                            target_card: card.id,
                                            play_id,
                                        },
                                    ))
                                    .anchor_card(card.id)
                            })
                            .collect(),
                    )
                }))
                .delegate(in_play::on_query_card_status_markers(|g, s, &card_id, mut markers| {
                    let card = g.card(s);
                    if card.is_last_target(card_id) {
                        markers.push(CardStatusMarker {
                            source: s.ability_id(),
                            marker_kind: CardInfoElementKind::NegativeEffect,
                            text: text![CardName::SpearOfUltimatum, "target"],
                        });
                    }
                    markers
                }))
                .build(),
            Ability::new_with_delegate(
                text!["Use this weapon only on chosen minion"],
                this::can_use_weapon(|g, s, encounter, flag| {
                    flag.add_constraint(g.card(s).is_last_target(encounter.minion_id))
                }),
            ),
            Ability::new(text![
                abilities::encounter_ability_text(
                    text![EncounterBoostCost],
                    text![EncounterBoostBonus]
                ),
                text![Breach]
            ]),
        ],
        config: CardConfigBuilder::new()
            .base_attack(2)
            .attack_boost(AttackBoost::new().mana_cost(2).bonus(3))
            .breach(meta.upgrade(1, 3))
            .resonance(Resonance::infernal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(26))
                    .fire_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Projectiles03"))
                    .impact_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Impact03")),
            )
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles1(4))
                    .scale(1.5)
                    .sound(SoundEffect::WaterMagic("RPG3_WaterMagicEpic_WaveImpact01"))
                    .effect_color(design::BLUE_500),
            )
            .build(),
    }
}

pub fn maul_of_devastation(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::MaulOfDevastation,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::riftcaller_card(meta, "maul_of_devastation"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Uncommon,
        abilities: vec![
            Ability::new(text![SlowAbility]),
            Ability::new(text!["When you access a room, this weapon loses slow until end of turn"])
                .delegate(this::is_slow_weapon(|g, _, _, _| {
                    history::rooms_accessed_this_turn(g).next().is_none()
                }))
                .delegate(in_play::on_raid_success(|g, s, _| {
                    if history::rooms_accessed_this_turn(g).next().is_none() {
                        visual_effects::show(g, s, s.card_id(), ShowAlert::No);
                    }
                    Ok(())
                })),
            abilities::encounter_boost(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(meta.upgrade(1, 2))
            .attack_boost(AttackBoost::new().mana_cost(1).bonus(1))
            .resonance(Resonance::mortal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles2(5))
                    .fire_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Projectiles01"))
                    .impact_sound(SoundEffect::WaterMagic("RPG3_WaterMagicEpic_Impact01")),
            )
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles1(9))
                    .scale(1.5)
                    .sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Cast03"))
                    .effect_color(design::BLUE_500),
            )
            .build(),
    }
}

pub fn amaras_decree(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::AmarasDecree,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(2),
        image: assets::riftcaller_card(meta, "amaras_decree"),
        card_type: CardType::Artifact,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Uncommon,
        abilities: abilities::some(vec![
            meta.is_upgraded.then(|| {
                Ability::new(text_helpers::named_trigger(Play, text!["Draw a card"])).delegate(
                    this::on_played(|g, s, _| {
                        draw_cards::run(g, Side::Riftcaller, 1, s.initiated_by())
                    }),
                )
            }),
            Some(
                Ability::new(text!["The Covenant cannot score schemes on the turn they’re played"])
                    .delegate(in_play::can_covenant_score_scheme(|g, s, &card_id, flag| {
                        flag.add_constraint(
                            history::cards_played_this_turn(g).all(|id| id != card_id),
                            s,
                        )
                    }))
                    .delegate(this::on_leaves_play(|g, _, _| {
                        mutations::check_for_covenant_scoring_schemes(g)
                    }))
                    .delegate(in_play::at_dusk(|g, _, _| {
                        mutations::check_for_covenant_scoring_schemes(g)
                    })),
            ),
        ]),
        config: CardConfigBuilder::new()
            .note(
                "Schemes which meet their progress requirement are scored on the subsequent \
                turn at Dusk",
            )
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles2(13))
                    .scale(1.5)
                    .effect_color(design::YELLOW_900)
                    .sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_Debuff_P1")),
            )
            .build(),
    }
}
