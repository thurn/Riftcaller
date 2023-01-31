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

//! Generates world maps for the 'adventure' game mode

pub mod card_generator;
pub mod mock_adventure;

use game_data::adventure::{AdventureConfiguration, AdventureState, Coins, TileEntity};

pub const STARTING_COINS: Coins = Coins(500);

/// Builds a new random 'adventure' mode world map
pub fn new_adventure(mut config: AdventureConfiguration) -> AdventureState {
    let collection = decklists::BASIC_CHAMPION.clone().cards;
    let explore = TileEntity::Explore { region: 2, cost: Coins(100) };
    let draft =
        TileEntity::Draft { cost: Coins(25), data: card_generator::draft_choices(&mut config) };
    let shop = TileEntity::Shop { data: card_generator::shop_options(&mut config) };
    mock_adventure::create(
        config,
        decklists::BASIC_CHAMPION.clone(),
        collection,
        Some(explore),
        Some(draft),
        Some(shop),
    )
}
