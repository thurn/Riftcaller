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

//! Contains the definitions for all cards in the game.

use assets::rexard_images;
use assets::rexard_images::RexardPack;
use card_helpers::identity_cost;
use data::card_definition::{CardConfig, CardDefinition};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};

pub mod artifacts;
pub mod canonical_game;
pub mod champion_identities;
pub mod champion_spells;
pub mod minions;
pub mod overlord_identities;
pub mod overlord_spells;
pub mod projects;
pub mod schemes;
pub mod weapons;

pub fn no_identity(name: CardName, side: Side, school: School) -> CardDefinition {
    CardDefinition {
        name,
        sets: vec![],
        cost: identity_cost(),
        image: rexard_images::get(RexardPack::MonstersAvatars, "22"),
        card_type: CardType::Identity,
        side,
        school,
        rarity: Rarity::Common,
        abilities: vec![],
        config: CardConfig::default(),
    }
}
