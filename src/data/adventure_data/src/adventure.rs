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

use std::collections::HashMap;

use anyhow::Result;
use core_data::adventure_primitives::{AdventureOutcome, CardFilterId, Coins, TilePosition};
use core_data::game_primitives::{AdventureId, Side};
use game_data::card_name::CardVariant;
use game_data::card_set_name::CardSetName;
use game_data::deck::Deck;
use game_data::player_name::{AIPlayer, PlayerId};
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::prelude::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_xoshiro::Xoshiro256StarStar;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use with_error::WithError;

use crate::adventure_effect_data::{AdventureEffect, DeckCardEffect};
use crate::narrative_event_data::NarrativeEventState;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CardChoice {
    pub quantity: u32,
    pub card: CardVariant,
    pub cost: Coins,
    pub sold: bool,
}

/// Contextual information about why the draft screen is being shown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DraftContext {
    StartingIdentity,
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
    /// Identifies the AI to use for this battle
    pub opponent_id: AIPlayer,

    /// Deck opponent will use for battle
    pub opponent_deck: Deck,

    /// Name displayed in the battle panel
    pub opponent_name: String,

    /// Coins earned for winning this battle
    pub reward: Coins,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TileIcon {
    Draft,
    Shop,
    Battle,
    NarrativeEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileState {
    pub sprite: String,

    pub road: Option<String>,

    pub on_visited: Option<AdventureEffect>,

    pub icons: Vec<TileIcon>,
}

impl TileState {
    pub fn with_sprite(address: impl Into<String>) -> Self {
        TileState { sprite: address.into(), road: None, on_visited: None, icons: vec![] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdventureConfiguration {
    pub player_id: PlayerId,
    /// Side the user is playing as in this adventure
    pub side: Side,
    /// Card set to draw cards from for this adventure
    pub card_set: CardSetName,
    /// Optionally, a random number generator for this adventure to use. This
    /// generator is serializable, so the state will be deterministic even
    /// across different sessions. If not specified, `rand::thread_rng()` is
    /// used instead and behavior is not deterministic.
    pub rng: Option<Xoshiro256StarStar>,
}

impl AdventureConfiguration {
    pub fn new(player_id: PlayerId, side: Side) -> Self {
        Self { player_id, side, card_set: CardSetName::Beryl, rng: None }
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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMap {
    /// Map from tile position to [TileState]
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub tiles: HashMap<TilePosition, TileState>,
}

impl WorldMap {
    /// Returns the [TileState] for a given tile position, or an error if no
    /// such tile position exists.
    pub fn tile(&self, position: TilePosition) -> Result<&TileState> {
        self.tiles.get(&position).with_error(|| format!("Tile not found {position:?}"))
    }

    /// Mutable version of [Self::tile].
    pub fn tile_mut(&mut self, position: TilePosition) -> Result<&mut TileState> {
        self.tiles.get_mut(&position).with_error(|| format!("Tile not found {position:?}"))
    }
}

/// Possible events/actions which can take place on a tile, represented by map
/// icons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdventureScreen {
    Draft(DraftData),
    Shop(ShopData),
    Battle(BattleData),
    NarrativeEvent(NarrativeEventState),
    ApplyDeckEffect(CardFilterId, DeckCardEffect),
}

/// Stores a stack of screens for events the player is interacting with in the
/// current adventure.
///
/// This is structured as a stack because screens sometimes show other screens,
/// e.g. a narrative event screen showing a draft choice.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdventureScreens {
    stack: Vec<AdventureScreen>,
}

impl AdventureScreens {
    /// Returns true if the player is not currently viewing any adventure
    /// screen.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Returns the screen the player is currently viewing, if any.
    pub fn current(&self) -> Option<&AdventureScreen> {
        self.stack.last()
    }

    /// Gets the [AdventureScreen] at the provided index
    pub fn get(&self, index: usize) -> Option<&AdventureScreen> {
        self.stack.get(index)
    }

    /// Mutable version of [Self::current].
    pub fn current_mut(&mut self) -> Option<&mut AdventureScreen> {
        self.stack.last_mut()
    }

    /// Number of screens currently stored
    pub fn count(&self) -> usize {
        self.stack.len()
    }

    /// Adds a new screen to the top of the stack
    pub fn push(&mut self, screen: AdventureScreen) {
        self.stack.push(screen);
    }

    /// Removes the topmost element from the adventure screen stack.
    pub fn pop(&mut self) {
        self.stack.pop();
    }
}

/// Stores the primary state for a player during an ongoing adventure
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdventureState {
    /// Unique identifier for this adventure
    pub id: AdventureId,
    /// Player type
    pub side: Side,
    /// Coin count, used to purchase more cards for deck
    pub coins: Coins,
    /// Result of the adventure, if it has ended.
    pub outcome: Option<AdventureOutcome>,
    /// States of world map tiles
    pub world_map: WorldMap,
    /// Stack of interstitial screens the player can view during an adventure.
    pub screens: AdventureScreens,
    /// Deck being used for this adventure
    pub deck: Deck,
    /// Customization options for this adventure
    pub config: AdventureConfiguration,
}
