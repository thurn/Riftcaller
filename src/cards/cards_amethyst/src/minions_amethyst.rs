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

//! Card definitions for the Minion card type

use assets::rexard_images;
use assets::rexard_images::RexardPack;
use card_helpers::text::trigger_text;
use card_helpers::{abilities, text, *};
use game_data::card_definition::{
    Ability, AbilityType, CardConfigBuilder, CardDefinition, Resonance,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardPosition;
use game_data::delegate_data::{Delegate, EventDelegate, RaidOutcome};
use game_data::game_actions::PromptChoice;
use game_data::game_effect::GameEffect;
use game_data::game_state::RaidJumpRequest;
use game_data::primitives::{CardType, InitiatedBy, Rarity, RoomLocation, School, Side};
use rules::mana::ManaPurpose;
use rules::mutations::SummonMinion;
use rules::{deal_damage, mana, mutations, queries, CardDefinitionExt};
use with_error::WithError;

pub fn time_golem(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TimeGolem,
        sets: vec![CardSetName::Amethyst],
        cost: cost(2),
        image: rexard_images::get(RexardPack::MonstersAvatars, "10"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            trigger_text(
                Encounter,
                text!["End the raid unless the Champion pays", Mana(5), "or", Actions(2)],
            ),
            on_encountered(|g, s, _| {
                show_prompt::with_choices(
                    g,
                    Side::Champion,
                    vec![
                        PromptChoice::new().effect(GameEffect::EndRaid(s.ability_id())),
                        PromptChoice::new().effect(GameEffect::ManaCost(
                            Side::Champion,
                            5,
                            s.initiated_by(),
                        )),
                        PromptChoice::new().effect(GameEffect::ActionCost(Side::Champion, 2)),
                    ],
                );
                Ok(())
            }),
        )],
        config: CardConfigBuilder::new().health(3).resonance(Resonance::infernal()).build(),
    }
}

pub fn temporal_stalker(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TemporalStalker,
        sets: vec![CardSetName::Amethyst],
        cost: cost(6),
        image: rexard_images::get(RexardPack::MonstersAvatars, "87"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                trigger_text(
                    Combat,
                    text!["Summon a minion from the", Sanctum, "or", Crypt, "for free"],
                ),
                combat(|g, s, _| {
                    let cards = g
                        .hand(Side::Overlord)
                        .chain(g.discard_pile(Side::Overlord))
                        .filter(|c| g.card(c.id).definition().card_type == CardType::Minion);
                    if let Some(minion_id) = queries::highest_cost(cards) {
                        let (room_id, index) = queries::minion_position(g, s.card_id())
                            .with_error(|| "Minion not found")?;
                        mutations::turn_face_down(g, minion_id); // Card may be face-up in Crypt
                        mutations::move_card(
                            g,
                            minion_id,
                            CardPosition::Room(room_id, RoomLocation::Defender),
                        )?;
                        g.move_card_to_index(minion_id, index);
                        mutations::summon_minion(
                            g,
                            minion_id,
                            s.initiated_by(),
                            SummonMinion::IgnoreCosts,
                        )?;
                        g.raid_mut()?.jump_request =
                            Some(RaidJumpRequest::EncounterMinion(minion_id));
                    }
                    Ok(())
                }),
            ),
            Ability::new_with_delegate(
                trigger_text(Combat, text!["End the raid unless the Champion pays", Actions(2)]),
                combat(|g, s, _| {
                    show_prompt::with_choices(
                        g,
                        Side::Champion,
                        vec![
                            PromptChoice::new().effect(GameEffect::EndRaid(s.ability_id())),
                            PromptChoice::new().effect(GameEffect::ActionCost(Side::Champion, 2)),
                        ],
                    );
                    Ok(())
                }),
            ),
        ],
        config: CardConfigBuilder::new().health(6).shield(3).resonance(Resonance::astral()).build(),
    }
}

pub fn shadow_lurker(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ShadowLurker,
        sets: vec![CardSetName::Amethyst],
        cost: cost(3),
        image: rexard_images::get(RexardPack::MonstersAvatars, "80"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text!["While this minion is in an", OuterRoom, "it has", Plus(2), Health],
                on_calculate_health(|g, s, _, current| match g.card(s.card_id()).position() {
                    CardPosition::Room(room_id, _) if !room_id.is_inner_room() => current + 2,
                    _ => current,
                }),
            ),
            abilities::combat_end_raid(),
        ],
        config: CardConfigBuilder::new().health(2).shield(1).resonance(Resonance::astral()).build(),
    }
}

pub fn sphinx_of_winters_breath(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SphinxOfWintersBreath,
        sets: vec![CardSetName::Amethyst],
        cost: cost(2),
        image: rexard_images::get(RexardPack::MonstersAvatars, "17"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: trigger_text(
                Combat,
                text![
                    text![DealDamage(1)],
                    text!["If a card with an odd mana cost is discarded, end the raid"]
                ],
            ),
            delegates: vec![
                combat(|g, s, _| deal_damage::apply(g, s, 1)),
                Delegate::DealtDamage(EventDelegate {
                    requirement: |g, s, data| {
                        let discarded = &g
                            .state_machines
                            .deal_damage
                            .as_ref()
                            .expect("Active damage event")
                            .discarded;

                        s.ability_id() == data.source
                            && discarded.iter().any(|card_id| {
                                queries::mana_cost(g, *card_id).unwrap_or(0) % 2 != 0
                            })
                    },
                    mutation: |g, s, _| {
                        mutations::end_raid(
                            g,
                            InitiatedBy::Ability(s.ability_id()),
                            RaidOutcome::Failure,
                        )
                    },
                }),
            ],
        }],
        config: CardConfigBuilder::new().health(3).shield(1).resonance(Resonance::mortal()).build(),
    }
}

pub fn bridge_troll(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::BridgeTroll,
        sets: vec![CardSetName::Amethyst],
        cost: cost(2),
        image: rexard_images::get(RexardPack::MonstersAvatars, "29"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            trigger_text(
                Combat,
                text![
                    text!["The Champion loses", Mana(3)],
                    text!["If they have", Mana(6), "or less, end the raid"]
                ],
            ),
            combat(|g, s, _| {
                mana::lose_upto(
                    g,
                    Side::Champion,
                    s.initiated_by(),
                    ManaPurpose::PayForTriggeredAbility,
                    3,
                );
                if mana::get(g, Side::Champion, ManaPurpose::BaseMana) <= 6 {
                    mutations::end_raid(
                        g,
                        InitiatedBy::Ability(s.ability_id()),
                        RaidOutcome::Failure,
                    )?;
                }
                Ok(())
            }),
        )],
        config: CardConfigBuilder::new().health(0).shield(2).resonance(Resonance::mortal()).build(),
    }
}

pub fn stormcaller(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Stormcaller,
        sets: vec![CardSetName::Amethyst],
        cost: cost(4),
        image: rexard_images::get(RexardPack::MonstersAvatars, "53"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            trigger_text(
                Combat,
                text![
                    text![DealDamage(2)],
                    text!["The Champion must end the raid or take 2 more damage"]
                ],
            ),
            combat(|g, s, _| {
                deal_damage::apply(g, s, 2)?;
                show_prompt::with_choices(
                    g,
                    Side::Champion,
                    vec![
                        PromptChoice::new().effect(GameEffect::EndRaid(s.ability_id())),
                        PromptChoice::new().effect(GameEffect::TakeDamageCost(s.ability_id(), 2)),
                    ],
                );
                Ok(())
            }),
        )],
        config: CardConfigBuilder::new()
            .health(3)
            .shield(2)
            .resonance(Resonance::infernal())
            .build(),
    }
}
