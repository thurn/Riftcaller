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

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::primitives::Side;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TilePosition {
    pub x: i32,
    pub y: i32,
}

/// Possible events/actions which can take place on a tile, represented by map
/// icons
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TileEntity {
    Draft,
    Explore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileState {
    pub sprite: String,

    pub road: Option<String>,

    pub entity: Option<TileEntity>,
}

impl TileState {
    pub fn with_sprite(address: impl Into<String>) -> Self {
        TileState { sprite: address.into(), road: None, entity: None }
    }
}

/// Stores the primary state for an ongoing game
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdventureState {
    pub side: Side,
    #[serde_as(as = "Vec<(_, _)>")]
    pub tiles: HashMap<TilePosition, TileState>,
}
