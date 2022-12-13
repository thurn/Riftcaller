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
use data::adventure::{AdventureState, AdventureStatus, TileEntity, TilePosition};
use data::player_data::PlayerData;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{LoadSceneCommand, SceneLoadMode};
use with_error::WithError;

pub fn handle_abandon_adventure(state: &mut AdventureState) -> Result<()> {
    state.status = AdventureStatus::Completed;
    Ok(())
}

pub fn handle_leave_adventure(state: &mut PlayerData) -> Result<Vec<Command>> {
    state.adventure = None;
    Ok(vec![Command::LoadScene(LoadSceneCommand {
        scene_name: "Main".to_string(),
        mode: SceneLoadMode::Single.into(),
        skip_if_current: true,
    })])
}

pub fn handle_tile_action(state: &mut AdventureState, position: TilePosition) -> Result<()> {
    let tile = state.tiles.get_mut(&position).with_error(|| "Tile not found")?;

    match tile.entity.with_error(|| "No action for tile")? {
        TileEntity::Explore { region, cost } => {
            state.coins -= cost;
            state.revealed_regions.insert(region);
            tile.entity = None;
        }
    }

    Ok(())
}
