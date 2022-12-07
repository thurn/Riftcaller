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

//! Implements game rules for the 'adventure' deckbuilding/drafting game mode

use anyhow::Result;
use data::adventure::{AdventureState, TileEntity, TilePosition};
use with_error::WithError;

pub fn handle_adventure_action(state: &mut AdventureState, position: TilePosition) -> Result<()> {
    let tile = state.tiles.get_mut(&position).with_error(|| "Tile not found")?;

    match tile.entity.with_error(|| "No action for tile")? {
        TileEntity::Draft => {}
        TileEntity::Explore(region_id) => {
            state.revealed_regions.insert(region_id);
            tile.entity = None;
        }
    }

    Ok(())
}
