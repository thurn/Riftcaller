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

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use core_data::adventure_primitives::{
    AdventureOutcome, Coins, NarrativeChoiceIndex, RegionId, Skill, TilePosition,
};
use core_data::game_primitives::{AdventureId, CardSubtype, CardType, Rarity, Side};
use game_data::card_name::CardVariant;
use game_data::character_preset::{CharacterFacing, CharacterPreset};
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

use crate::adventure_effect::AdventureEffectData;

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

    /// Opponent character appearance
    pub character: CharacterPreset,

    /// Direction character is facing
    pub character_facing: CharacterFacing,

    /// Map region which should be revealed if the player wins this battle.
    pub region_to_reveal: RegionId,
}

/// One possible choice within a narrative event screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEventChoice {
    /// Narrative description of the action for this choice.
    pub choice_description: String,
    /// Narrative description of the outcome of this choice.
    pub result_description: String,
    /// Skill required to select this choice, if any.
    pub skill: Option<Skill>,
    /// Costs to select this choice.
    ///
    /// Choices will not be presented unless the player is able to pay all of
    /// their associated costs.
    pub costs: Vec<AdventureEffectData>,
    /// Effect of selecting this choice.
    pub effects: Vec<AdventureEffectData>,
}

/// Steps within the progress of resolving a narrative event.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum NarrativeEventStep {
    /// Introductory text for this event.
    Introduction,
    /// View valid narrative choices for this event which have not yet been
    /// selected.
    ViewChoices,
    /// View the result of selecting the narrative choice with the provided
    /// [NarrativeChoiceIndex].
    SelectChoice(NarrativeChoiceIndex),
}

/// Data for displaying a narrative event to the player with a fixed set of
/// choices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEventData {
    /// Current screen within the event.
    pub step: NarrativeEventStep,
    /// Narrative description introducing this event.
    pub description: String,
    /// List of possible choices within this narrative event, indexed by
    /// [NarrativeChoiceIndex].
    pub choices: Vec<NarrativeEventChoice>,
    /// Indices of [Self::choices] which the player has selected, in order of
    /// selection.
    ///
    /// In almost all cases, the player only selects a single choice within a
    /// narrative event, but a few situations exist where multiple options may
    /// be selected.
    pub selected_choices: Vec<NarrativeChoiceIndex>,
}

impl NarrativeEventData {
    pub fn enumerate_choices(
        &self,
    ) -> impl Iterator<Item = (NarrativeChoiceIndex, &NarrativeEventChoice)> {
        self.choices
            .iter()
            .enumerate()
            .map(|(value, choice)| (NarrativeChoiceIndex { value }, choice))
    }

    pub fn choice(&self, index: NarrativeChoiceIndex) -> &NarrativeEventChoice {
        &self.choices[index.value]
    }
}

/// Possible events/actions which can take place on a tile, represented by map
/// icons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileEntity {
    Draft(DraftData),
    Shop(ShopData),
    Battle(BattleData),
    NarrativeEvent(NarrativeEventData),
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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMap {
    /// Map from tile position to [TileState]
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub tiles: HashMap<TilePosition, TileState>,
    /// Current tile entity position on the world map which the player is
    /// visiting. If not specified, the player is not currently visiting any
    /// tile.
    pub visiting_position: Option<TilePosition>,
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

    /// Returns the [TileEntity] the player is currently visiting, or an error
    /// if no such tile entity exists.
    pub fn visiting_tile(&self) -> Result<&TileEntity> {
        self.tile(self.visited_position()?)?.entity.as_ref().with_error(|| "Expected tile entity")
    }

    pub fn visiting_tile_option(&self) -> Option<&TileEntity> {
        self.tiles.get(&self.visiting_position?)?.entity.as_ref()
    }

    /// Mutable version of [Self::visiting_tile].
    pub fn visiting_tile_mut(&mut self) -> Result<&mut TileEntity> {
        self.tile_mut(self.visited_position()?)?
            .entity
            .as_mut()
            .with_error(|| "Expected tile entity")
    }

    /// Removes the entity from the currently-visited tile and clears the
    /// visiting position.
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
    /// Result of the adventure, if it has ended.
    pub outcome: Option<AdventureOutcome>,
    /// States of world map tiles
    pub world_map: WorldMap,
    /// Regions which the player can currently see. By default Region 1 is
    /// revealed.
    pub revealed_regions: HashSet<RegionId>,
    /// Deck being used for this adventure
    pub deck: Deck,
    /// Cards collected by this player during this adventure, inclusive of cards
    /// in `deck` and cards not currently being used.
    #[serde_as(as = "Vec<(_, _)>")]
    pub collection: HashMap<CardVariant, u32>,
    /// Customization options for this adventure
    pub config: AdventureConfiguration,
}

/// Specifies the parameters for picking a card from a set
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardSelector {
    /// Minimum rarity for cards
    pub rarity: Option<Rarity>,
    /// Card types to select from.
    ///
    /// If empty, all card types are allowed.
    pub card_types: Vec<CardType>,
    /// Card subtypes to select from.
    ///
    /// If empty, all card subtypes are allowed.    
    pub card_subtypes: Vec<CardSubtype>,
}

impl CardSelector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rarity(mut self, rarity: Rarity) -> Self {
        self.rarity = Some(rarity);
        self
    }

    pub fn card_type(mut self, card_type: CardType) -> Self {
        self.card_types.push(card_type);
        self
    }

    pub fn card_subtype(mut self, card_subtype: CardSubtype) -> Self {
        self.card_subtypes.push(card_subtype);
        self
    }
}
