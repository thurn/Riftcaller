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

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use anyhow::Result;
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Sub, SubAssign, Sum,
};
use serde::{Deserialize, Serialize};
use with_error::fail;

/// Identifies a set of tiles which can be revealed via the 'explore' action.
pub type RegionId = u32;

#[derive(
    Debug,
    Display,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum AdventureOutcome {
    Victory,
    Defeat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TilePosition {
    pub x: i32,
    pub y: i32,
}

impl FromStr for TilePosition {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let vec = s.split(',').collect::<Vec<_>>();
        if vec.len() == 2 {
            Ok(TilePosition { x: vec[0].parse::<i32>()?, y: vec[1].parse::<i32>()? })
        } else {
            fail!("Expected exactly one ',' character")
        }
    }
}

impl Display for TilePosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl TilePosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
