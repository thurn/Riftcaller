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

pub mod adventure_over_panel;
pub mod adventure_panels;
pub mod draft_panel;
pub mod draft_prompt_panel;
pub mod explore_panel;
pub mod full_screen_image_panel;
pub mod tile_prompt_panel;

use anyhow::Result;
use core_ui::prelude::*;
use core_ui::{actions, design, panel};
use data::adventure::{AdventureScreen, AdventureState, TileEntity, TilePosition, TileState};
use panel_address::PanelAddress;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    FlexVector3, MapTileType, SpriteAddress, UpdateWorldMapCommand, WorldMapSprite, WorldMapTile,
};

use crate::adventure_over_panel::AdventureOverPanel;
use crate::draft_panel::DraftPanel;

/// Returns a sequence of game Commands to display the provided
/// [AdventureState].
pub fn render(state: &AdventureState) -> Result<Vec<Command>> {
    let mut commands = vec![Command::UpdateWorldMap(UpdateWorldMapCommand {
        tiles: state
            .tiles
            .iter()
            .filter(|(_, tile)| state.revealed_regions.contains(&tile.region_id))
            .map(|(position, state)| render_tile(*position, state))
            .collect(),
    })];

    if state.screen.is_some() {
        let screen = render_adventure_screen(state);
        commands.push(panel::update(PanelAddress::AdventureScreen, screen));
        commands.push(panel::open(PanelAddress::AdventureScreen));
    }

    Ok(commands)
}

/// Renders a screen based on the [AdventureScreen] contained within the
/// provided state, if any
pub fn render_adventure_screen(state: &AdventureState) -> Option<Node> {
    match &state.screen {
        Some(AdventureScreen::AdventureOver) => AdventureOverPanel::new().build(),
        Some(AdventureScreen::Draft(data)) => DraftPanel { data }.build(),
        _ => None,
    }
}

fn render_tile(position: TilePosition, tile: &TileState) -> WorldMapTile {
    let mut sprites = vec![WorldMapSprite {
        sprite_address: Some(SpriteAddress {
            address: format!("DavidBaumgart/WorldTiles.spriteatlas[{}]", tile.sprite),
        }),
        color: None,
        anchor_offset: None,
        scale: None,
    }];

    if let Some(road) = &tile.road {
        sprites.push(WorldMapSprite {
            sprite_address: Some(SpriteAddress {
                address: format!("DavidBaumgart/Roads.spriteatlas[{}]", road),
            }),
            ..WorldMapSprite::default()
        })
    }

    if let Some(entity) = &tile.entity {
        sprites.push(WorldMapSprite {
            sprite_address: Some(SpriteAddress {
                address: "Sprites/MapIconBackground.png".to_string(),
            }),
            color: Some(design::BLACK),
            anchor_offset: Some(FlexVector3 { x: 0.0, y: 1.28, z: 0.0 }),
            scale: Some(FlexVector3 { x: 0.6, y: 0.6, z: 1.0 }),
        });

        sprites.push(WorldMapSprite {
            sprite_address: Some(sprite_address_for_entity(*entity)),
            color: Some(design::WHITE),
            anchor_offset: Some(FlexVector3 { x: 0.0, y: 1.28, z: 0.0 }),
            scale: Some(FlexVector3 { x: 0.6, y: 0.6, z: 1.0 }),
        });
    }

    WorldMapTile {
        sprites,
        position: Some(adapters::map_position(position)),
        on_visit: tile
            .entity
            .map(|_| actions::client_action(panel::open(PanelAddress::TileEntity(position)))),
        tile_type: if tile.entity.is_some() {
            MapTileType::Visitable.into()
        } else if tile.road.is_some() {
            MapTileType::Walkable.into()
        } else {
            MapTileType::Obstacle.into()
        },
    }
}

fn sprite_address_for_entity(entity: TileEntity) -> SpriteAddress {
    SpriteAddress {
        address: match entity {
            TileEntity::Draft { .. } => {
                "RainbowArt/CleanFlatIcon/png_128/icon/icon_store/icon_store_167.png"
            }
            TileEntity::Explore { .. } => {
                "RainbowArt/CleanFlatIcon/png_128/icon/icon_app/icon_app_198.png"
            }
        }
        .to_string(),
    }
}
