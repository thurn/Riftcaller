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

use std::collections::HashMap;

use data::adventure::{AdventureState, TileEntity, TilePosition, TileState};
use data::primitives::Side;

/// Builds a new random 'adventure' mode world map
pub fn new_adventure(side: Side) -> AdventureState {
    let mut tiles = HashMap::new();

    add_tile(&mut tiles, -3, 2, "hexGrassySandPalms02");
    add_tile(&mut tiles, -2, 2, "hexGrassySandPalms03");
    add_tile(&mut tiles, -1, 2, "hexPlainsCold03");
    add_tile(&mut tiles, 0, 2, "hexMarsh00");
    add_tile(&mut tiles, 1, 2, "hexPlainsHalflingVillage00");
    add_tile(&mut tiles, 2, 2, "hexDirtInn00");
    add_with_road_and_entity(
        &mut tiles,
        3,
        2,
        "hexPlains00",
        "hexRoad-010010-01",
        TileEntity::Explore,
    );
    add_tile(&mut tiles, 4, 2, "hexPlainsSmithy00");
    add_tile(&mut tiles, -4, 1, "hexGrassySandPalms01");
    add_tile(&mut tiles, -3, 1, "hexPlainsFarm02");
    add_tile(&mut tiles, -2, 1, "hexWoodlands02");
    add_tile(&mut tiles, -1, 1, "hexHills02");
    add_tile(&mut tiles, 0, 1, "hexHills00");
    add_tile(&mut tiles, 1, 1, "hexHighlands02");
    add_with_road(&mut tiles, 2, 1, "hexScrublands01", "hexRoad-010010-00");
    add_tile(&mut tiles, 3, 1, "hexPlainsFarm01");
    add_tile(&mut tiles, -3, 0, "hexDirtCastle00");
    add_with_road(&mut tiles, -2, 0, "hexPlains00", "hexRoad-001010-00");
    add_with_road(&mut tiles, -1, 0, "hexPlains02", "hexRoad-001001-01");
    add_with_road(&mut tiles, 0, 0, "hexScrublands01", "hexRoad-001001-00");
    add_with_road(&mut tiles, 1, 0, "hexPlains01", "hexRoad-001001-00");
    add_with_road(&mut tiles, 2, 0, "hexPlains02", "hexRoad-010101-00");
    add_tile(&mut tiles, 3, 0, "hexWoodlands00");
    add_tile(&mut tiles, 4, 0, "hexJungle02");
    add_tile(&mut tiles, -4, -1, "hexPlainsFarm01");
    add_with_road(&mut tiles, -3, -1, "hexScrublands01", "hexRoad-010010-00");
    add_tile(&mut tiles, -2, -1, "hexTropicalPlains00");
    add_tile(&mut tiles, -1, -1, "hexSwamp01");
    add_with_entity(&mut tiles, 0, -1, "hexMountain03", TileEntity::Draft);
    add_tile(&mut tiles, 1, -1, "hexPlainsFarm00");
    add_with_road(&mut tiles, 2, -1, "hexPlains00", "hexRoad-100100-00");
    add_tile(&mut tiles, 3, -1, "hexJungle03");
    add_with_road(&mut tiles, -3, -2, "hexScrublands00", "hexRoad-010010-01");
    add_tile(&mut tiles, -2, -2, "hexForestBroadleafForester00");
    add_tile(&mut tiles, -1, -2, "hexSwamp00");
    add_tile(&mut tiles, 0, -2, "hexSwamp03");
    add_tile(&mut tiles, 1, -2, "hexForestBroadleaf00");
    add_tile(&mut tiles, 2, -2, "hexHills02");
    add_with_road(&mut tiles, 3, -2, "hexPlains00", "hexRoad-100100-01");
    add_tile(&mut tiles, 4, -2, "hexJungle00");

    AdventureState { side, tiles }
}

fn add_tile(map: &mut HashMap<TilePosition, TileState>, x: i32, y: i32, sprite: &'static str) {
    map.insert(TilePosition { x, y }, TileState::with_sprite(sprite));
}

fn add_with_road(
    map: &mut HashMap<TilePosition, TileState>,
    x: i32,
    y: i32,
    sprite: &'static str,
    road: &'static str,
) {
    // Road hex names are numbered clockwise from top-left
    map.insert(
        TilePosition { x, y },
        TileState { road: Some(road.to_string()), ..TileState::with_sprite(sprite) },
    );
}

fn add_with_entity(
    map: &mut HashMap<TilePosition, TileState>,
    x: i32,
    y: i32,
    sprite: &'static str,
    entity: TileEntity,
) {
    map.insert(
        TilePosition { x, y },
        TileState { entity: Some(entity), ..TileState::with_sprite(sprite) },
    );
}

fn add_with_road_and_entity(
    map: &mut HashMap<TilePosition, TileState>,
    x: i32,
    y: i32,
    sprite: &'static str,
    road: &'static str,
    entity: TileEntity,
) {
    map.insert(
        TilePosition { x, y },
        TileState {
            road: Some(road.to_string()),
            entity: Some(entity),
            ..TileState::with_sprite(sprite)
        },
    );
}
