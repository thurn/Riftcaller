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

//! Card definitions for the Spell card type & Champion player

use assets::rexard_images;
use card_helpers::abilities::standard;
use card_helpers::{text, *};
use game_data::card_definition::{
    Ability, AbilityType, CardConfig, CardConfigBuilder, CardDefinition, TargetRequirement,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::delegates::{Delegate, QueryDelegate};
use game_data::primitives::{CardType, Rarity, RoomId, School, Side};
use rules::{flags, mana, mutations, CardDefinitionExt};

pub fn meditation(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Meditation,
        sets: vec![CardSetName::Amethyst],
        cost: cost(1),
        image: rexard_images::spell(1, "SpellBook01_98"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![standard(
            text![text![Gain, Mana(5)], text![Lose, Actions(1), reminder("(if able)")]],
            this::on_play(|g, s, _| {
                mana::gain(g, s.side(), 5);
                mutations::lose_action_points_if_able(g, s.side(), 1)
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn coup_de_grace(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::CoupDeGrace,
        sets: vec![CardSetName::Amethyst],
        cost: cost(0),
        image: rexard_images::spell(1, "SpellBook01_76"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: text![
                text!["Raid the", Sanctum, "or", Vault, ", accessing", 1, "additional card"],
                text!["If successful, draw a card"]
            ],
            delegates: vec![
                this::on_play(|g, s, play_card| initiate_raid(g, s, play_card.target)),
                add_vault_access::<1>(matching_raid),
                add_sanctum_access::<1>(matching_raid),
                on_raid_success(matching_raid, |g, s, _| {
                    mutations::draw_cards(g, s.side(), 1).map(|_| ())
                }),
            ],
        }],
        config: CardConfigBuilder::new()
            .custom_targeting(TargetRequirement::TargetRoom(|game, _, room_id| {
                flags::can_take_initiate_raid_action(game, Side::Champion, room_id)
                    && (room_id == RoomId::Sanctum || room_id == RoomId::Vault)
            }))
            .build(),
    }
}

pub fn charged_strike(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ChargedStrike,
        sets: vec![CardSetName::Amethyst],
        cost: cost(1),
        image: rexard_images::spell(1, "SpellBook01_67"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![standard(
            text![text![BeginARaid], text![Gain, Mana(5), "to spend during that raid"]],
            this::on_play(|g, s, play_card| {
                initiate_raid_with_callback(g, s, play_card.target, |game, raid_id| {
                    mana::add_raid_specific_mana(game, s.side(), raid_id, 5);
                })
            }),
        )],
        config: CardConfigBuilder::new()
            .custom_targeting(TargetRequirement::TargetRoom(|game, _, room_id| {
                flags::can_take_initiate_raid_action(game, Side::Champion, room_id)
            }))
            .build(),
    }
}

pub fn stealth_mission(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::StealthMission,
        sets: vec![CardSetName::Amethyst],
        cost: cost(1),
        image: rexard_images::spell(1, "SpellBook01_89"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: text![
                text![BeginARaid],
                text!["During that raid, summon costs are increased by", Mana(3)]
            ],
            delegates: vec![
                this::on_play(|g, s, play_card| initiate_raid(g, s, play_card.target)),
                Delegate::ManaCost(QueryDelegate {
                    requirement: matching_raid,
                    transformation: |g, _s, card_id, current| {
                        if g.card(*card_id).definition().card_type == CardType::Minion {
                            current.map(|current| current + 3)
                        } else {
                            current
                        }
                    },
                }),
            ],
        }],
        config: CardConfigBuilder::new()
            .custom_targeting(TargetRequirement::TargetRoom(|game, _, room_id| {
                flags::can_take_initiate_raid_action(game, Side::Champion, room_id)
            }))
            .build(),
    }
}

pub fn preparation(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Preparation,
        sets: vec![CardSetName::Amethyst],
        cost: cost(1),
        image: rexard_images::spell(1, "SpellBook01_79"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![standard(
            text![text!["Draw", 4, "cards"], text!["Lose", Actions(1), reminder("(if able)")]],
            this::on_play(|g, s, _| {
                mutations::draw_cards(g, s.side(), 4)?;
                mutations::lose_action_points_if_able(g, s.side(), 1)
            }),
        )],
        config: CardConfig::default(),
    }
}
