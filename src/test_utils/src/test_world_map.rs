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

use std::collections::HashMap;

use core_data::adventure_primitives::TilePosition;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::WorldMapTile;

#[derive(Default, Clone)]
pub struct TestWorldMap {
    tiles: HashMap<(i32, i32), TestMapTile>,
}

impl TestWorldMap {
    pub fn update(&mut self, command: Command) {
        if let Command::UpdateWorldMap(map) = command {
            for tile in map.tiles {
                let clone = tile.clone();
                let position = tile.position.expect("tile position").clone();
                self.tiles.insert((position.x, position.y), TestMapTile { tile: clone });
            }
        }
    }

    pub fn tile(&self, position: TilePosition) -> &TestMapTile {
        self.tiles.get(&(position.x, position.y)).expect("Tile not found")
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    pub fn find_tile_with_sprite(&self, substring: impl Into<String>) -> &TestMapTile {
        let pattern = substring.into();
        self.tiles
            .values()
            .find(move |tile| tile.has_sprite(&pattern))
            .expect("Matching tile not found")
    }
}

#[derive(Clone)]
pub struct TestMapTile {
    pub tile: WorldMapTile,
}

impl TestMapTile {
    pub fn has_sprite(&self, substring: &str) -> bool {
        self.tile.sprites.iter().any(|sprite| {
            sprite.sprite_address.as_ref().expect("sprite_address").address.contains(substring)
        })
    }
}
