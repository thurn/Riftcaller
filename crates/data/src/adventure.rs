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

use anyhow::Result;
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Sub, SubAssign, Sum,
};
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::prelude::IteratorRandom;
use rand::Rng;
use rand_xoshiro::Xoshiro256StarStar;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use with_error::WithError;

use crate::card_name::CardName;
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum AdventureStatus {
    InProgress,
    Completed,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardChoice {
    pub quantity: u32,
    pub card: CardName,
    pub cost: Coins,
}

/// Data for rendering the draft screen
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DraftData {
    pub choices: Vec<CardChoice>,
}

/// Data for rendering the shop screen
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShopData {
    pub choices: Vec<CardChoice>,
}

/// Possible events/actions which can take place on a tile, represented by map
/// icons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileEntity {
    Explore { region: RegionId, cost: Coins },
    Draft { cost: Coins, data: DraftData },
    Shop { data: ShopData },
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

/// Represents an active choice screen within an adventure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdventureChoiceScreen {
    /// Adventure has ended
    AdventureOver,
    /// Pick one card of a set of draft options
    Draft(TilePosition),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdventureConfiguration {
    /// Side the user is playing as in this adventure
    pub side: Side,
    /// Optionally, a random number generator for this adventure to use. This
    /// generator is serializable, so the state will be deterministic even
    /// across different sessions. If not specified, `rand::thread_rng()` is
    /// used instead and behavior is not deterministic.
    pub rng: Option<Xoshiro256StarStar>,
}

impl AdventureConfiguration {
    pub fn new(side: Side) -> Self {
        Self { side, rng: None }
    }

    pub fn choose<I>(&mut self, iterator: I) -> Option<I::Item>
    where
        I: Iterator,
    {
        if self.rng.is_some() {
            iterator.choose_stable(self.rng.as_mut().expect("rng"))
        } else {
            iterator.choose(&mut rand::thread_rng())
        }
    }

    pub fn choose_multiple<I>(&mut self, amount: usize, iterator: I) -> Vec<I::Item>
    where
        I: Iterator,
    {
        if self.rng.is_some() {
            iterator.choose_multiple(self.rng.as_mut().expect("rng"), amount)
        } else {
            iterator.choose_multiple(&mut rand::thread_rng(), amount)
        }
    }

    pub fn gen_range<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        if self.rng.is_some() {
            self.rng.as_mut().expect("rng").gen_range(range)
        } else {
            rand::thread_rng().gen_range(range)
        }
    }
}

/// Stores the primary state for one player during an ongoing adventure
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdventureState {
    /// Player type
    pub side: Side,
    /// Coin count, used to purchase more cards for deck
    pub coins: Coins,
    /// Currently active mandatory choice screen, if any.
    pub choice_screen: Option<AdventureChoiceScreen>,
    /// States of world map tiles
    #[serde_as(as = "Vec<(_, _)>")]
    pub tiles: HashMap<TilePosition, TileState>,
    /// Regions which the player can currently see. By default Region 1 is
    /// revealed.
    pub revealed_regions: HashSet<RegionId>,
    /// Cards collected by this player during this adventure
    #[serde_as(as = "Vec<(_, _)>")]
    pub collection: HashMap<CardName, u32>,
    /// Customization options for this adventure
    pub config: AdventureConfiguration,
}

impl AdventureState {
    /// Returns the [TileState] for a given tile position, or an error if no
    /// such tile position exists.
    pub fn tile(&self, position: TilePosition) -> Result<&TileState> {
        self.tiles.get(&position).with_error(|| "Tile not found")
    }

    /// Mutable version of [Self::tile].
    pub fn tile_mut(&mut self, position: TilePosition) -> Result<&mut TileState> {
        self.tiles.get_mut(&position).with_error(|| "Tile not found")
    }

    /// Returns the [TileEntity] for a given tile position, or an error if no
    /// such tile entity exists.
    pub fn tile_entity(&self, position: TilePosition) -> Result<&TileEntity> {
        self.tile(position)?.entity.as_ref().with_error(|| "Expected tile entity")
    }
}
