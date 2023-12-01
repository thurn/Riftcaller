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

//! Generates world maps for the 'adventure' game mode

use adventure_data::adventure::{AdventureConfiguration, AdventureState, TileEntity};
use core_data::adventure_primitives::Coins;
use core_data::game_primitives::{AdventureId, Side};

pub mod battle_generator;
pub mod card_generator;
pub mod mock_adventure;

pub const STARTING_COINS: Coins = Coins(500);

/// Builds a new random 'adventure' mode world map
pub fn new_adventure(mut config: AdventureConfiguration) -> AdventureState {
    let side = config.side;
    let deck = match side {
        Side::Overlord => decklists::BASIC_OVERLORD.clone(),
        Side::Champion => decklists::BASIC_CHAMPION.clone(),
    };
    let riftcallers = TileEntity::Draft(card_generator::riftcaller_choices(&mut config));
    let draft = TileEntity::Draft(card_generator::draft_choices(&mut config));
    let shop = TileEntity::Shop(card_generator::shop_options(&mut config));
    let battle = TileEntity::Battle(battle_generator::create(side.opponent()));
    mock_adventure::create(
        AdventureId::generate(),
        config,
        deck.clone(),
        deck.cards,
        Some(riftcallers),
        Some(draft),
        Some(shop),
        Some(battle),
    )
}
