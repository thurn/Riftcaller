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

//! Card definitions for the Spell card type & Riftcaller player

use assets::rexard_images;
use card_helpers::raids::{add_sanctum_access, add_vault_access};
use card_helpers::*;
use core_data::game_primitives::{CardType, InitiatedBy, Rarity, RoomId, School, Side};
use game_data::card_definition::{
    Ability, AbilityType, CardConfig, CardConfigBuilder, CardDefinition, TargetRequirement,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::delegate_data::{Delegate, QueryDelegate};
use raid_state::InitiateRaidOptions;
use rules::{draw_cards, flags, mana, mutations, CardDefinitionExt};

pub fn meditation(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Meditation,
        sets: vec![CardSetName::Amethyst],
        cost: cost(1),
        image: rexard_images::spell(1, "SpellBook01_98"),
        card_type: CardType::Spell,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![text![GainMana(5)], text![Lose, Actions(1), "if able"]],
            this::on_played(|g, s, _| {
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
        card_type: CardType::Spell,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: text![
                text!["Raid the", Sanctum, "or", Vault, ", accessing", 1, "additional card"],
                text!["If successful, draw a card"]
            ],
            delegates: vec![
                this::on_played(|g, s, play_card| raids::initiate(g, s, play_card.target)),
                add_vault_access::<1>(requirements::matching_raid),
                add_sanctum_access::<1>(requirements::matching_raid),
                on_raid_success(requirements::matching_raid, |g, s, _| {
                    draw_cards::run(g, s.side(), 1, s.initiated_by()).map(|_| ())
                }),
            ],
        }],
        config: CardConfigBuilder::new()
            .custom_targeting(TargetRequirement::TargetRoom(|g, _, room_id| {
                flags::is_valid_raid_target(g, room_id)
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
        card_type: CardType::Spell,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![text![BeginARaid], text![GainMana(5), "to spend during that raid"]],
            this::on_played(|g, s, play_card| {
                raid_state::initiate_with_callback(
                    g,
                    play_card.target.room_id()?,
                    InitiatedBy::Ability(s.ability_id()),
                    InitiateRaidOptions::default(),
                    |game, raid_id| {
                        mana::add_raid_specific_mana(game, s.side(), raid_id, 5);
                    },
                )
            }),
        )],
        config: CardConfigBuilder::new().custom_targeting(requirements::any_raid_target()).build(),
    }
}

pub fn stealth_mission(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::StealthMission,
        sets: vec![CardSetName::Amethyst],
        cost: cost(1),
        image: rexard_images::spell(1, "SpellBook01_89"),
        card_type: CardType::Spell,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: text![
                text![BeginARaid],
                text!["During that raid, summon costs are increased by", Mana(3)]
            ],
            delegates: vec![
                this::on_played(|g, s, play_card| {
                    card_helpers::raids::initiate(g, s, play_card.target)
                }),
                Delegate::ManaCost(QueryDelegate {
                    requirement: requirements::matching_raid,
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
        config: CardConfigBuilder::new().custom_targeting(requirements::any_raid_target()).build(),
    }
}

pub fn preparation(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Preparation,
        sets: vec![CardSetName::Amethyst],
        cost: cost(1),
        image: rexard_images::spell(1, "SpellBook01_79"),
        card_type: CardType::Spell,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![text!["Draw", 4, "cards"], text!["Lose", Actions(1), "if able"]],
            this::on_played(|g, s, _| {
                draw_cards::run(g, s.side(), 4, s.initiated_by())?;
                mutations::lose_action_points_if_able(g, s.side(), 1)
            }),
        )],
        config: CardConfig::default(),
    }
}
