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

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use anyhow::Result;
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Sub, SubAssign, Sum,
};
use serde::{Deserialize, Serialize};
use with_error::fail;

use crate::game_primitives::AbilityIndex;

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

/// Unique identifier for a card filter
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct CardFilterId {
    pub value: u32,
}

impl CardFilterId {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

/// Unique identifier for a narrative event
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct NarrativeEventId {
    pub value: u32,
}

impl NarrativeEventId {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

/// Identifies a choice index within a narrative event.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct NarrativeChoiceId {
    pub value: u32,
}

impl NarrativeChoiceId {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

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

/// Skills which a Riftcaller can use to overcome narrative encounters
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub enum Skill {
    Brawn,
    Stealth,
    Lore,
    Persuasion,
}

/// Identifies a card with abilities which modify game events during an
/// adventure.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub enum AdventureCardId {
    /// An identity card within the player's deck at a given index position.
    ///
    /// Currently only one identity card is allowed.
    Identity(usize),
    /// A sigil card within the player's deck at a given index position.
    Sigil(usize),
    /// A global Game Modifier card within the adventure's active modifier list.
    GameModifier(usize),
}

/// Identifies an ability within a card in the context of an adventure.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct AdventureAbilityId {
    pub card_id: AdventureCardId,
    pub index: AbilityIndex,
}
