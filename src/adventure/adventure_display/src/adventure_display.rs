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

//! Implements rendering for the 'adventure' deckbuilding/drafting game mode

use adventure_data::adventure::{AdventureState, TileEntity, TileState};
use adventure_data::adventure_action::AdventureAction;
use anyhow::Result;
use core_data::adventure_primitives::TilePosition;
use core_ui::actions::InterfaceAction;
use core_ui::design;
use core_ui::panels::Panels;
use panel_address::{PanelAddress, PlayerPanel};
use protos::riftcaller::game_command::Command;
use protos::riftcaller::{
    FlexVector3, InterfacePanel, MapTileType, SpriteAddress, UpdateWorldMapCommand,
    WorldMapCharacter, WorldMapSprite, WorldMapTile,
};

pub mod adventure_over_panel;
pub mod adventure_panels;
pub mod battle_panel;
pub mod draft_panel;
pub mod narrative_event_panel;
pub mod shop_panel;

/// Returns a sequence of game Commands to display the provided
/// [AdventureState].
pub fn render(state: &AdventureState) -> Result<Vec<Command>> {
    let mut commands = vec![Command::UpdateWorldMap(UpdateWorldMapCommand {
        tiles: state
            .world_map
            .tiles
            .iter()
            .filter(|(_, tile)| state.revealed_regions.contains(&tile.region_id))
            .map(|(position, state)| render_tile(*position, state))
            .collect(),
    })];

    if let Some(_) = &state.outcome {
        commands.push(Panels::open(PlayerPanel::AdventureOver).into());
    } else if let Some(position) = state.world_map.visiting_position {
        commands.push(Panels::open(PlayerPanel::AdventureTile(position)).into());
    }

    Ok(commands)
}

pub struct RenderedChoiceScreen {
    pub panel: Option<InterfacePanel>,
    pub address: PanelAddress,
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

    let mut character = None;
    if let Some(entity) = &tile.entity {
        if let Some(sprite) = sprite_address_for_entity(entity) {
            sprites.push(WorldMapSprite {
                sprite_address: Some(SpriteAddress {
                    address: "Sprites/MapIconBackground.png".to_string(),
                }),
                color: Some(design::BLACK),
                anchor_offset: Some(FlexVector3 { x: 0.0, y: 1.28, z: 0.0 }),
                scale: Some(FlexVector3 { x: 0.6, y: 0.6, z: 1.0 }),
            });

            sprites.push(WorldMapSprite {
                sprite_address: Some(sprite),
                color: Some(design::WHITE),
                anchor_offset: Some(FlexVector3 { x: 0.0, y: 1.28, z: 0.0 }),
                scale: Some(FlexVector3 { x: 0.6, y: 0.6, z: 1.0 }),
            });
        }

        if let TileEntity::Battle(battle) = entity {
            character = Some(WorldMapCharacter {
                appearance: Some(assets::character_preset(battle.character)),
                facing_direction: adapters::game_character_facing_direction(
                    battle.character_facing,
                ),
            });
        }
    }

    WorldMapTile {
        sprites,
        position: Some(adapters::map_position(position)),
        on_visit: tile.entity.as_ref().map(|_| {
            Panels::open(PlayerPanel::AdventureTile(position))
                .action(AdventureAction::VisitTileEntity(position))
                .build()
        }),
        tile_type: if tile.entity.is_some() {
            MapTileType::Visitable.into()
        } else if tile.road.is_some() {
            MapTileType::Walkable.into()
        } else {
            MapTileType::Obstacle.into()
        },
        character,
    }
}

fn sprite_address_for_entity(entity: &TileEntity) -> Option<SpriteAddress> {
    let address = match entity {
        TileEntity::Draft(_) => {
            Some("RainbowArt/CleanFlatIcon/png_128/icon/icon_store/icon_store_167.png".to_string())
        }
        TileEntity::Shop(_) => Some(
            "RainbowArt/CleanFlatIcon/png_128/icon/icon_architecture/icon_architecture_6.png"
                .to_string(),
        ),
        TileEntity::Battle(_) => None,
        TileEntity::NarrativeEvent(_) => {
            Some("RainbowArt/CleanFlatIcon/png_128/icon/icon_game/icon_game_194.png".to_string())
        }
    };

    address.map(|a| SpriteAddress { address: a })
}
