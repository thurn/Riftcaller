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

//! Card definitions for the Spell card type & Covenant player

use assets::rexard_images;
use card_helpers::*;
use core_data::game_primitives::{CardType, Rarity, School, Side};
use game_data::card_definition::{
    Ability, CardConfig, CardConfigBuilder, CardDefinition, TargetRequirement,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use rules::{flags, mana, mutations};

pub fn overwhelming_power(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::OverwhelmingPower,
        sets: vec![CardSetName::Amethyst],
        cost: cost(10),
        image: rexard_images::spell(1, "SpellBook01_92"),
        card_type: CardType::Ritual,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![GainMana(15)],
            this::on_played(|g, s, _| {
                mana::gain(g, s.side(), 15);
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn forced_march(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ForcedMarch,
        sets: vec![CardSetName::Amethyst],
        cost: cost(1),
        image: rexard_images::spell(3, "SpellBook03_04"),
        card_type: CardType::Ritual,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![
                "Place 2 progress counters on each card in target room which didn't enter play this turn"
            ],
            this::on_played(|g, s, played| {
                mutations::progress_card_occupying_room(
                    g,
                    played.target.room_id()?,
                    s.initiated_by(),
                    2)
            }),
        )],
        config: CardConfigBuilder::new().custom_targeting(
            TargetRequirement::TargetRoom(|game, _, room_id| {
                game.defenders_and_occupants(room_id).any(|card| {
                    flags::can_progress_card(game, card.id)
                        && !history::played_this_turn(game, card.id)
                })
            })).build()
    }
}
