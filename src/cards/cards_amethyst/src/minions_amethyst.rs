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
use card_helpers::{abilities, text, *};
use game_data::card_definition::{Ability, AbilityType, CardConfigBuilder, CardDefinition};
use game_data::card_name::CardName;
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardPosition;
use game_data::delegates::{Delegate, EventDelegate, RaidOutcome};
use game_data::game::RaidJumpRequest;
use game_data::primitives::{CardType, Rarity, Resonance, RoomLocation, School, Side};
use rules::mana::ManaPurpose;
use rules::mutations::SummonMinion;
use rules::{mana, mutations, queries};
use with_error::WithError;

pub fn time_golem() -> CardDefinition {
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
        abilities: vec![
            abilities::construct(),
            simple_ability(
                trigger_text(
                    Encounter,
                    text!["End the raid unless the Champion pays", Mana(5), "or", Actions(2)],
                ),
                on_encountered(|g, _s, _| {
                    mutations::add_card_prompt(
                        g,
                        Side::Champion,
                        vec![
                            end_raid_prompt(),
                            lose_mana_prompt(g, Side::Champion, 5),
                            lose_actions_prompt(g, Side::Champion, 2),
                        ],
                    )
                }),
            ),
        ],
        config: CardConfigBuilder::new().health(3).resonance(Resonance::Infernal).build(),
    }
}

pub fn temporal_stalker() -> CardDefinition {
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
            simple_ability(
                trigger_text(
                    Combat,
                    text!["Summon a minion from the", Sanctum, "or", Crypts, "for free"],
                ),
                combat(|g, s, _| {
                    let cards = g
                        .hand(Side::Overlord)
                        .chain(g.discard_pile(Side::Overlord))
                        .filter(|c| rules::card_definition(g, c.id).card_type == CardType::Minion);
                    if let Some(minion_id) = queries::highest_cost(cards) {
                        let (room_id, index) = queries::minion_position(g, s.card_id())
                            .with_error(|| "Minion not found")?;
                        g.card_mut(minion_id).turn_face_down(); // Card may be face-up in Crypt
                        mutations::move_card(
                            g,
                            minion_id,
                            CardPosition::Room(room_id, RoomLocation::Defender),
                        )?;
                        g.move_card_to_index(minion_id, index);
                        mutations::summon_minion(g, minion_id, SummonMinion::IgnoreCosts)?;
                        g.raid_mut()?.jump_request =
                            Some(RaidJumpRequest::EncounterMinion(minion_id));
                    }
                    Ok(())
                }),
            ),
            simple_ability(
                trigger_text(Combat, text!["End the raid unless the Champion pays", Actions(2)]),
                combat(|g, _, _| {
                    mutations::add_card_prompt(
                        g,
                        Side::Champion,
                        vec![end_raid_prompt(), lose_actions_prompt(g, Side::Champion, 2)],
                    )
                }),
            ),
        ],
        config: CardConfigBuilder::new().health(6).shield(3).resonance(Resonance::Abyssal).build(),
    }
}

pub fn shadow_lurker() -> CardDefinition {
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
            simple_ability(
                text!["While this minion is in an", OuterRoom, "it has", Plus(2), Health],
                on_calculate_health(|g, s, _, current| match g.card(s.card_id()).position() {
                    CardPosition::Room(room_id, _) if !is_inner_room(room_id) => current + 2,
                    _ => current,
                }),
            ),
            abilities::combat_end_raid(),
        ],
        config: CardConfigBuilder::new().health(2).shield(1).resonance(Resonance::Abyssal).build(),
    }
}

pub fn sphinx_of_winters_breath() -> CardDefinition {
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
                combat(|g, s, _| mutations::deal_damage(g, s, 1)),
                Delegate::DealtDamage(EventDelegate {
                    requirement: |g, s, data| {
                        s.ability_id() == data.source
                            && data.discarded.iter().any(|card_id| {
                                queries::mana_cost(g, *card_id).unwrap_or(0) % 2 != 0
                            })
                    },
                    mutation: |g, _, _| mutations::end_raid(g, RaidOutcome::Failure),
                }),
            ],
        }],
        config: CardConfigBuilder::new().health(3).shield(1).resonance(Resonance::Mortal).build(),
    }
}

pub fn bridge_troll() -> CardDefinition {
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
        abilities: vec![simple_ability(
            trigger_text(
                Combat,
                text![
                    text!["The Champion loses", Mana(3)],
                    text!["If they have", Mana(6), "or less, end the raid"]
                ],
            ),
            combat(|g, _, _| {
                mana::lose_upto(g, Side::Champion, ManaPurpose::PayForTriggeredAbility, 3);
                if mana::get(g, Side::Champion, ManaPurpose::BaseMana) <= 6 {
                    mutations::end_raid(g, RaidOutcome::Failure)?;
                }
                Ok(())
            }),
        )],
        config: CardConfigBuilder::new().health(0).shield(2).resonance(Resonance::Mortal).build(),
    }
}

pub fn stormcaller() -> CardDefinition {
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
        abilities: vec![simple_ability(
            trigger_text(
                Combat,
                text![
                    text![DealDamage(2)],
                    text!["The Champion must end the raid or take 2 more damage"]
                ],
            ),
            combat(|g, s, _| {
                mutations::deal_damage(g, s, 2)?;
                mutations::add_card_prompt(
                    g,
                    Side::Champion,
                    vec![take_damage_prompt(g, s, 2), end_raid_prompt()],
                )
            }),
        )],
        config: CardConfigBuilder::new().health(3).shield(2).resonance(Resonance::Infernal).build(),
    }
}
