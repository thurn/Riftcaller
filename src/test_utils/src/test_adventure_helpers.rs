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

use std::sync::atomic::{AtomicI32, Ordering};

use adventure_data::adventure::TileState;
use adventure_data::adventure_effect_data::AdventureEffect;
use core_data::adventure_primitives::TilePosition;

use crate::test_session::TestSession;
use crate::TestSessionHelpers;

pub static NEXT_X_POSITION: AtomicI32 = AtomicI32::new(1);

pub trait TestAdventureHelpers {
    fn insert_tile(&mut self, effect: AdventureEffect) -> TilePosition;

    fn insert_tile_at_position(&mut self, effect: AdventureEffect, position: TilePosition);

    fn visit_tile(&mut self, position: TilePosition);
}

impl TestAdventureHelpers for TestSession {
    fn insert_tile(&mut self, effect: AdventureEffect) -> TilePosition {
        let position = TilePosition::new(NEXT_X_POSITION.fetch_add(1, Ordering::SeqCst), 1);
        self.insert_tile_at_position(effect, position);
        position
    }

    fn insert_tile_at_position(&mut self, effect: AdventureEffect, position: TilePosition) {
        self.overwrite_adventure_tile(
            position,
            TileState {
                sprite: "/sprite.png".to_string(),
                road: None,
                on_visited: Some(effect),
                icons: vec![],
            },
        );
    }

    fn visit_tile(&mut self, position: TilePosition) {
        let tile = self.client.map.tile(position);
        let action = tile.tile.on_visit.as_ref().expect("No visit action found");
        self.perform(action.action.as_ref().expect("action").clone(), self.user_id())
    }
}
