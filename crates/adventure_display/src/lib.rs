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
use core_ui::design;
use data::adventure::{AdventureState, TileEntity, TilePosition, TileState};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    FlexVector3, MapTileType, SpriteAddress, UpdateWorldMapCommand, WorldMapTile,
};

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
        position: Some(adapters::map_position(position)),
        z_index: 0,
        tile_type: if tile.entity.is_some() {
            MapTileType::Visitable.into()
        } else if tile.road.is_some() {
            MapTileType::Walkable.into()
        } else {
            MapTileType::Obstacle.into()
        },
        color: None,
        anchor_offset: None,
        scale: None,
    }];

    if let Some(road) = &tile.road {
        result.push(WorldMapTile {
            sprite_address: Some(SpriteAddress {
                address: format!("DavidBaumgart/Roads.spriteatlas[{}]", road),
            }),
            position: Some(adapters::map_position(position)),
            z_index: 1,
            color: None,
            anchor_offset: None,
            scale: None,
            ..WorldMapTile::default()
        })
    }

    if let Some(entity) = &tile.entity {
        result.push(WorldMapTile {
            sprite_address: Some(SpriteAddress {
                address: "Sprites/MapIconBackground.png".to_string(),
            }),
            position: Some(adapters::map_position(position)),
            z_index: 2,
            color: Some(design::BLACK),
            anchor_offset: Some(FlexVector3 { x: 0.0, y: 1.28, z: 0.0 }),
            scale: Some(FlexVector3 { x: 0.6, y: 0.6, z: 1.0 }),
            ..WorldMapTile::default()
        });

        result.push(WorldMapTile {
            sprite_address: Some(sprite_address_for_entity(*entity)),
            position: Some(adapters::map_position(position)),
            z_index: 3,
            color: Some(design::WHITE),
            anchor_offset: Some(FlexVector3 { x: 0.0, y: 1.28, z: 0.0 }),
            scale: Some(FlexVector3 { x: 0.6, y: 0.6, z: 1.0 }),
            ..WorldMapTile::default()
        });
    }

    result
}

fn sprite_address_for_entity(entity: TileEntity) -> SpriteAddress {
    SpriteAddress {
        address: match entity {
            TileEntity::Draft => {
                "RainbowArt/CleanFlatIcon/png_128/icon/icon_store/icon_store_167.png"
            }
            TileEntity::Explore => {
                "RainbowArt/CleanFlatIcon/png_128/icon/icon_app/icon_app_198.png"
            }
        }
        .to_string(),
    }
}
