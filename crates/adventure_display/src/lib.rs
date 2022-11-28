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

//! Implements rendering for the 'adventure' deckbuilding/drafting game mode

use anyhow::Result;
use data::adventure::{AdventureState, TilePosition, TileState};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{SpriteAddress, UpdateWorldMapCommand, WorldMapTile};

/// Returns a sequence of game Commands to display the provided
/// [AdventureState].
pub fn render(state: &AdventureState) -> Result<Vec<Command>> {
    Ok(vec![Command::UpdateWorldMap(UpdateWorldMapCommand {
        tiles: state
            .tiles
            .iter()
            .flat_map(|(position, state)| render_tile(*position, state))
            .collect(),
    })])
}

fn render_tile(position: TilePosition, tile: &TileState) -> Vec<WorldMapTile> {
    let mut result = vec![WorldMapTile {
        sprite_address: Some(SpriteAddress {
            address: format!("DavidBaumgart/WorldTiles.spriteatlas[{}]", tile.sprite),
        }),
        x: position.x,
        y: position.y,
        z: 0,
    }];

    if let Some(road) = &tile.road {
        result.push(WorldMapTile {
            sprite_address: Some(SpriteAddress {
                address: format!("DavidBaumgart/Roads.spriteatlas[{}]", road),
            }),
            x: position.x,
            y: position.y,
            z: 1,
        })
    }

    result
}
