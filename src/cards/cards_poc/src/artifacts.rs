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
use assets::rexard_images::{RexardArtifactType, RexardPack};
use card_helpers::{abilities, text, *};
use game_data::card_definition::{
    Ability, AbilityType, CardConfig, CardDefinition, Cost, TargetRequirement,
};
use game_data::card_name::CardName;
use game_data::card_set_name::CardSetName;
use game_data::delegates::{Delegate, EventDelegate};
use game_data::primitives::{CardType, Rarity, School, Side};
use game_data::text::trigger;
use game_data::utils;
use rules::mutations;
use rules::mutations::OnZeroStored;

pub fn invisibility_ring() -> CardDefinition {
    CardDefinition {
        name: CardName::InvisibilityRing,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(2),
        image: rexard_images::get(RexardPack::JeweleryRings, "rn_b_03"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: text![
                "The first time each turn you access the Sanctum, access",
                1,
                "additional card"
            ],
            delegates: vec![
                on_raid_access_start(face_up_in_play, |g, s, raid_id| {
                    once_per_turn(g, s, raid_id, save_raid_id)
                }),
                add_sanctum_access::<1>(matching_raid),
            ],
        }],
        config: CardConfig::default(),
    }
}

pub fn accumulator() -> CardDefinition {
    CardDefinition {
        name: CardName::Accumulator,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(3),
        image: rexard_images::get(RexardPack::JeweleryNecklaces, "07_ob"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            simple_ability(
                trigger(SuccessfulRaid, text![StoreMana(1)]),
                on_raid_success(face_up_in_play, |g, s, _| {
                    add_stored_mana(g, s.card_id(), 1);
                    alert(g, s);
                    Ok(())
                }),
            ),
            Ability {
                ability_type: activate_for_action(),
                text: text![StoreMana(1), ", then take all stored", ManaSymbol],
                delegates: vec![on_activated(|g, s, activated| {
                    let mana = add_stored_mana(g, s.card_id(), 1);
                    mutations::take_stored_mana(g, activated.card_id(), mana, OnZeroStored::Ignore)
                        .map(|_| ())
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn mage_gloves() -> CardDefinition {
    CardDefinition {
        name: CardName::MageGloves,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(5),
        image: rexard_images::artifact(RexardArtifactType::Gloves, "gloves_20"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana_on_play::<12>(),
            Ability {
                ability_type: AbilityType::Activated(
                    actions(1),
                    TargetRequirement::TargetRoom(|g, _, room_id| {
                        is_inner_room(room_id)
                            && utils::is_false(|| {
                                Some(g.room_state.get(&room_id)?.last_raided? == g.data.turn)
                            })
                    }),
                ),
                text: text![
                    text!["Raid an", InnerRoom, "you have not raided this turn"],
                    text!["If successful,", TakeMana(3)]
                ],
                delegates: vec![
                    on_activated(|g, s, activated| initiate_raid(g, s, activated.target)),
                    on_raid_success(matching_raid, |g, s, _| {
                        mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)
                            .map(|_| ())
                    }),
                ],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn magical_resonator() -> CardDefinition {
    CardDefinition {
        name: CardName::MagicalResonator,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(1),
        image: rexard_images::artifact(RexardArtifactType::Bracers, "bracers_2"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana_on_play::<9>(),
            Ability {
                ability_type: AbilityType::Activated(
                    Cost { mana: None, actions: 1, custom_cost: once_per_turn_cost() },
                    TargetRequirement::None,
                ),
                text: text![text![TakeMana(3)], text!["Use this ability once per turn"]],
                delegates: vec![on_activated(|g, _s, activated| {
                    mutations::take_stored_mana(g, activated.card_id(), 3, OnZeroStored::Sacrifice)
                        .map(|_| ())
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn dark_grimoire() -> CardDefinition {
    CardDefinition {
        name: CardName::DarkGrimoire,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(3),
        image: rexard_images::get(RexardPack::MagicItems, "book_06_b"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!["The first time each turn you take the 'draw card' action, draw another card"],
            Delegate::DrawCardAction(EventDelegate {
                requirement: face_up_in_play,
                mutation: |g, s, _| {
                    once_per_turn(g, s, &(), |g, s, _| {
                        mutations::draw_cards(g, s.side(), 1).map(|_| ())
                    })
                },
            }),
        )],
        config: CardConfig::default(),
    }
}
