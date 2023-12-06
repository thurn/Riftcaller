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

use assets::rexard_images;
use assets::rexard_images::{RexardArtifactType, RexardPack};
use card_helpers::costs::{actions, once_per_turn};
use card_helpers::raids::add_sanctum_access;
use card_helpers::requirements::FaceUpInPlay;
use card_helpers::this::on_activated;
use card_helpers::{abilities, *};
use core_data::game_primitives::{CardType, Rarity, School, Side};
use game_data::card_definition::{
    Ability, AbilityType, CardConfig, CardDefinition, Cost, TargetRequirement,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::delegate_data::{Delegate, EventDelegate};
use rules::mutations::{add_stored_mana, OnZeroStored};
use rules::visual_effects::VisualEffects;
use rules::{draw_cards, mutations};

pub fn invisibility_ring(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::InvisibilityRing,
        sets: vec![CardSetName::Amethyst],
        cost: cost(2),
        image: rexard_images::get(RexardPack::JeweleryRings, "rn_b_03"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: text![
                "The first time each turn you access the Sanctum, access",
                1,
                "additional card"
            ],
            delegates: vec![add_sanctum_access::<1>(
                requirements::no_sanctum_access::<FaceUpInPlay>,
            )],
        }],
        config: CardConfig::default(),
    }
}

pub fn accumulator(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Accumulator,
        sets: vec![CardSetName::Amethyst],
        cost: cost(3),
        image: rexard_images::get(RexardPack::JeweleryNecklaces, "07_ob"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text!["When you access a room", StoreMana(1)],
                in_play::on_raid_success(|g, s, _| {
                    VisualEffects::new().ability_alert(s).apply(g);
                    add_stored_mana(g, s.card_id(), 1);
                    Ok(())
                }),
            ),
            Ability {
                ability_type: activate_for_action(),
                text: text![StoreMana(1), ", then take all stored", ManaSymbol],
                delegates: vec![on_activated(|g, s, activated| {
                    let mana = add_stored_mana(g, s.card_id(), 1);
                    mutations::take_stored_mana(
                        g,
                        activated.card_id(),
                        mana,
                        OnZeroStored::Ignore,
                    )?;
                    Ok(())
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn mage_gloves(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::MageGloves,
        sets: vec![CardSetName::Amethyst],
        cost: cost(5),
        image: rexard_images::artifact(RexardArtifactType::Gloves, "gloves_20"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana_on_play::<12>(),
            Ability {
                ability_type: AbilityType::Activated {
                    cost: actions(1),
                    target_requirement: TargetRequirement::TargetRoom(|g, _, room_id| {
                        room_id.is_inner_room()
                            & &history::rooms_raided_this_turn(g).all(|r| r != room_id)
                    }),
                },
                text: text![
                    text!["Raid an", InnerRoom, "you have not raided this turn"],
                    text!["If successful,", TakeMana(3)]
                ],
                delegates: vec![
                    on_activated(|g, s, activated| {
                        card_helpers::raids::initiate(g, s, activated.target)
                    }),
                    on_raid_success(requirements::matching_raid, |g, s, _| {
                        mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)?;
                        Ok(())
                    }),
                ],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn magical_resonator(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::MagicalResonator,
        sets: vec![CardSetName::Amethyst],
        cost: cost(1),
        image: rexard_images::artifact(RexardArtifactType::Bracers, "bracers_2"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana_on_play::<9>(),
            Ability {
                ability_type: AbilityType::Activated {
                    cost: Cost { mana: None, actions: 1, custom_cost: once_per_turn() },
                    target_requirement: TargetRequirement::None,
                },
                text: text![text![TakeMana(3)], text!["Use this ability once per turn"]],
                delegates: vec![on_activated(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)?;
                    Ok(())
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn dark_grimoire(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::DarkGrimoire,
        sets: vec![CardSetName::Amethyst],
        cost: cost(3),
        image: rexard_images::get(RexardPack::MagicItems, "book_06_b"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![
                "The first time each turn you take the 'draw card' action,
draw another card"
            ],
            Delegate::DrawCardAction(EventDelegate {
                requirement: requirements::no_card_draw_actions::<FaceUpInPlay>,
                mutation: |g, s, _| {
                    draw_cards::run(g, s.side(), 1, s.initiated_by())?;
                    Ok(())
                },
            }),
        )],
        config: CardConfig::default(),
    }
}
