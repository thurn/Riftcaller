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

use std::collections::{HashMap, HashSet};

use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Sub, SubAssign, Sum,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::primitives::Side;

/// Identifies a set of tiles which can be revealed via the 'explore' action.
pub type RegionId = u32;

#[derive(
    Debug,
    Display,
    Copy,
    Clone,
    PartialEq,
    Eq,
    From,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Into,
    Serialize,
    Deserialize,
)]
pub struct Coins(pub u32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TilePosition {
    pub x: i32,
    pub y: i32,
}

impl TilePosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Possible events/actions which can take place on a tile, represented by map
/// icons
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TileEntity {
    Explore { region: RegionId, cost: Coins },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileState {
    pub sprite: String,

    pub road: Option<String>,

    pub entity: Option<TileEntity>,

    pub region_id: RegionId,
}

impl TileState {
    pub fn with_sprite(address: impl Into<String>) -> Self {
        TileState { sprite: address.into(), road: None, entity: None, region_id: 1 }
    }
}

/// Stores the primary state for an ongoing adventure
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdventureState {
    pub side: Side,
    pub coins: Coins,
    /// States of world map tiles
    #[serde_as(as = "Vec<(_, _)>")]
    pub tiles: HashMap<TilePosition, TileState>,
    /// Regions which the player can currently see. By default Region 1 is
    /// revealed.
    pub revealed_regions: HashSet<RegionId>,
}
