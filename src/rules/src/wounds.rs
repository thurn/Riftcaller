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

use anyhow::Result;
use core_data::game_primitives::{HasAbilityId, WoundCount};
use game_data::game_state::GameState;

/// Give the Champion player `quantity` wounds, which decreases their maximum
/// hand size.
pub fn give_wounds(game: &mut GameState, _: impl HasAbilityId, quantity: WoundCount) -> Result<()> {
    game.champion.wounds += quantity;
    Ok(())
}

/// Remove `quantity` wounds from the Champion player.
pub fn remove_wounds(game: &mut GameState, quantity: WoundCount) -> Result<()> {
    game.champion.wounds = game.champion.wounds.saturating_sub(quantity);
    Ok(())
}
