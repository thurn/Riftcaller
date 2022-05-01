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

use data::card_definition::{Ability, CardConfig, CardDefinition, CustomTargeting};
use data::card_name::CardName;
use data::delegates::RaidOutcome;
use data::primitives::{CardType, Rarity, RoomId, School, Side};
use linkme::distributed_slice;
use rules::card_text::text;
use rules::helpers::*;
use rules::{mana, mutations, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn arcane_recovery() -> CardDefinition {
    CardDefinition {
        name: CardName::ArcaneRecovery,
        cost: cost(5),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_25"),
        card_type: CardType::Spell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text!("Gain", mana(9)),
            ability_type: silent(),
            delegates: vec![on_cast(|g, s, _| mana::gain(g, s.side(), 9))],
        }],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn meditation() -> CardDefinition {
    CardDefinition {
        name: CardName::Meditation,
        cost: cost(1),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_24"),
        card_type: CardType::Spell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text!("Gain", mana(5), ".", "Lose", actions(1), reminder("(if able).")),
            ability_type: silent(),
            delegates: vec![on_cast(|g, s, _| {
                mana::gain(g, s.side(), 5);
                mutations::lose_action_point_if_able(g, s.side(), 1);
            })],
        }],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn coup_de_grace() -> CardDefinition {
    CardDefinition {
        name: CardName::CoupDeGrace,
        cost: cost(0),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_26"),
        card_type: CardType::Spell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text!(
                "Raid the Sanctum or Vault, accessing 1 additional card.",
                "If successful, draw a card."
            ),
            ability_type: silent(),
            delegates: vec![
                on_cast(|g, s, play_card| initiate_raid(g, s, play_card.target)),
                add_vault_access::<1>(matching_raid),
                add_sanctum_access::<1>(matching_raid),
                on_raid_ended(matching_raid, |g, s, raid_ended| {
                    if raid_ended.outcome == RaidOutcome::Success {
                        mutations::draw_cards(g, s.side(), 1);
                    }
                }),
            ],
        }],
        config: CardConfig {
            custom_targeting: Some(CustomTargeting::TargetRoom(|r| {
                r == RoomId::Sanctum || r == RoomId::Vault
            })),
            ..CardConfig::default()
        },
    }
}
