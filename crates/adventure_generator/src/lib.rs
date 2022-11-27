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

use data::adventure::{AdventureState, TilePosition, TileState};
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
    add_tile(&mut tiles, 3, 2, "hexPlains00");
    add_tile(&mut tiles, 4, 2, "hexPlainsSmithy00");
    add_tile(&mut tiles, -4, 1, "hexGrassySandPalms01");
    add_tile(&mut tiles, -3, 1, "hexPlainsFarm02");
    add_tile(&mut tiles, -2, 1, "hexWoodlands02");
    add_tile(&mut tiles, -1, 1, "hexScrublands01");
    add_tile(&mut tiles, 0, 1, "hexHills00");
    add_tile(&mut tiles, 1, 1, "hexHighlands02");
    add_tile(&mut tiles, 2, 1, "hexScrublands01");
    add_tile(&mut tiles, 3, 1, "hexPlainsFarm01");
    add_tile(&mut tiles, -3, 0, "hexDirtCastle00");
    add_tile(&mut tiles, -2, 0, "hexPlains00");
    add_tile(&mut tiles, -1, 0, "hexPlains02");
    add_tile(&mut tiles, 0, 0, "hexScrublands01");
    add_tile(&mut tiles, 1, 0, "hexPlains01");
    add_tile(&mut tiles, 2, 0, "hexPlains02");
    add_tile(&mut tiles, 3, 0, "hexWoodlands00");
    add_tile(&mut tiles, 4, 0, "hexJungle02");
    add_tile(&mut tiles, -4, -1, "hexPlains02");
    add_tile(&mut tiles, -3, -1, "hexScrublands01");
    add_tile(&mut tiles, -2, -1, "hexTropicalPlains00");
    add_tile(&mut tiles, -1, -1, "hexSwamp01");
    add_tile(&mut tiles, 0, -1, "hexDirtVillage01");
    add_tile(&mut tiles, 1, -1, "hexPlainsFarm00");
    add_tile(&mut tiles, 2, -1, "hexPlains00");
    add_tile(&mut tiles, 3, -1, "hexJungle03");
    add_tile(&mut tiles, -3, -2, "hexScrublands00");
    add_tile(&mut tiles, -2, -2, "hexForestBroadleafForester00");
    add_tile(&mut tiles, -1, -2, "hexSwamp00");
    add_tile(&mut tiles, 0, -2, "hexSwamp03");
    add_tile(&mut tiles, 1, -2, "hexForestBroadleaf00");
    add_tile(&mut tiles, 2, -2, "hexHills02");
    add_tile(&mut tiles, 3, -2, "hexPlains00");
    add_tile(&mut tiles, 4, -2, "hexJungle00");

    AdventureState { side, tiles }
}

fn add_tile(map: &mut HashMap<TilePosition, TileState>, x: i32, y: i32, sprite: &'static str) {
    map.insert(TilePosition { x, y }, TileState { sprite: sprite.to_string() });
}
