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
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use anyhow::Result;
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Sub, SubAssign, Sum,
};
use game_data::card_name::CardName;
use game_data::character_preset::{CharacterFacing, CharacterPreset};
use game_data::deck::Deck;
use game_data::player_name::PlayerId;
use game_data::primitives::{AdventureId, Side};
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::prelude::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_xoshiro::Xoshiro256StarStar;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use with_error::{fail, WithError};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CardChoice {
    pub quantity: u32,
    pub card: CardName,
    pub cost: Coins,
    pub sold: bool,
}

/// Contextual information about why the draft screen is being shown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DraftContext {
    StartingSigil,
}

/// Data for rendering the draft screen
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DraftData {
    pub context: Option<DraftContext>,
    pub choices: Vec<CardChoice>,
}

/// Data for rendering the shop screen
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShopData {
    pub choices: Vec<CardChoice>,
}

/// Data for rendering an opponent character to initiate a battle with
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleData {
    /// Deck opponent will use for battle
    pub opponent_deck: Deck,

    /// Opponent character appearance
    pub character: CharacterPreset,

    /// Direction character is facing
    pub character_facing: CharacterFacing,
}

/// Possible events/actions which can take place on a tile, represented by map
/// icons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileEntity {
    Draft(DraftData),
    Shop(ShopData),
    Battle(BattleData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileState {
    pub sprite: String,

    pub road: Option<String>,

    pub entity: Option<TileEntity>,

    pub region_id: RegionId,

    pub visited: bool,
}

impl TileState {
    pub fn with_sprite(address: impl Into<String>) -> Self {
        TileState { sprite: address.into(), road: None, entity: None, region_id: 1, visited: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdventureConfiguration {
    pub player_id: PlayerId,
    /// Side the user is playing as in this adventure
    pub side: Side,
    /// Optionally, a random number generator for this adventure to use. This
    /// generator is serializable, so the state will be deterministic even
    /// across different sessions. If not specified, `rand::thread_rng()` is
    /// used instead and behavior is not deterministic.
    pub rng: Option<Xoshiro256StarStar>,
}

impl AdventureConfiguration {
    pub fn new(player_id: PlayerId, side: Side) -> Self {
        Self { player_id, side, rng: None }
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
            let rng = self.rng.as_mut().expect("rng");
            let mut result = iterator.choose_multiple(rng, amount);
            result.shuffle(rng);
            result
        } else {
            let mut result = iterator.choose_multiple(&mut rand::thread_rng(), amount);
            result.shuffle(&mut rand::thread_rng());
            result
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
    /// Unique identifier for this adventure
    pub id: AdventureId,
    /// Player type
    pub side: Side,
    /// Coin count, used to purchase more cards for deck
    pub coins: Coins,
    /// Current tile entity position on the world map which the player is
    /// visiting. If not specified, the player is not currently visiting any
    /// tile.
    pub visiting_position: Option<TilePosition>,
    /// Result of the adventure, if it has ended.
    pub outcome: Option<AdventureOutcome>,
    /// States of world map tiles
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub tiles: HashMap<TilePosition, TileState>,
    /// Regions which the player can currently see. By default Region 1 is
    /// revealed.
    pub revealed_regions: HashSet<RegionId>,
    /// Deck being used for this adventure
    pub deck: Deck,
    /// Cards collected by this player during this adventure, inclusive of cards
    /// in `deck` and cards not currently being used.
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub collection: HashMap<CardName, u32>,
    /// Customization options for this adventure
    pub config: AdventureConfiguration,
}

impl AdventureState {
    /// Returns the [TileState] for a given tile position, or an error if no
    /// such tile position exists.
    pub fn tile(&self, position: TilePosition) -> Result<&TileState> {
        self.tiles.get(&position).with_error(|| format!("Tile not found {position:?}"))
    }

    /// Mutable version of [Self::tile].
    pub fn tile_mut(&mut self, position: TilePosition) -> Result<&mut TileState> {
        self.tiles.get_mut(&position).with_error(|| format!("Tile not found {position:?}"))
    }

    /// Returns the [TileEntity] the player is currently visiting, or an error
    /// if no such tile entity exists.
    pub fn visiting_tile(&self) -> Result<&TileEntity> {
        self.tile(self.visited_position()?)?.entity.as_ref().with_error(|| "Expected tile entity")
    }

    /// Mutable version of [Self::visiting_tile].
    pub fn visiting_tile_mut(&mut self) -> Result<&mut TileEntity> {
        self.tile_mut(self.visited_position()?)?
            .entity
            .as_mut()
            .with_error(|| "Expected tile entity")
    }

    /// Removes the entity from the currently-visited tile and clears the
    /// visting position.
    pub fn clear_visited_tile(&mut self) -> Result<()> {
        self.tile_mut(self.visited_position()?)?.entity = None;
        self.visiting_position = None;
        Ok(())
    }

    /// Returns the tile position currently being visited, or an error if no
    /// such position exists.
    fn visited_position(&self) -> Result<TilePosition> {
        self.visiting_position.with_error(|| "Expected visited tile")
    }
}
