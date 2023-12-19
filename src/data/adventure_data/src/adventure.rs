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
use core_data::adventure_primitives::{
    AdventureOutcome, Coins, NarrativeChoiceIndex, Skill, TilePosition,
};
use core_data::game_primitives::{AdventureId, CardSubtype, CardType, Rarity, Side};
use enumset::EnumSet;
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

use crate::adventure_action::NarrativeEffectIndex;
use crate::adventure_effect_data::{AdventureEffect, AdventureEffectData};

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
    /// Rewards for selecting this choice.
    pub rewards: Vec<AdventureEffectData>,
    /// Effects of this choice which the user has already applied and thus do
    /// not need to be shown on-screen.
    pub applied: Vec<NarrativeEffectIndex>,
}

impl NarrativeEventChoice {
    pub fn effect(&self, index: NarrativeEffectIndex) -> &AdventureEffectData {
        match index {
            NarrativeEffectIndex::Cost(i) => &self.costs[i],
            NarrativeEffectIndex::Reward(i) => &self.rewards[i],
        }
    }

    pub fn enumerate_costs(
        &self,
    ) -> impl Iterator<Item = (NarrativeEffectIndex, &AdventureEffectData)> {
        self.costs.iter().enumerate().map(|(i, choice)| (NarrativeEffectIndex::Cost(i), choice))
    }

    pub fn enumerate_rewards(
        &self,
    ) -> impl Iterator<Item = (NarrativeEffectIndex, &AdventureEffectData)> {
        self.rewards.iter().enumerate().map(|(i, choice)| (NarrativeEffectIndex::Reward(i), choice))
    }

    /// Returns true if all of the costs and rewards for this choice have either
    /// 1) been applied or 2) are immediate and thus do not need to be applied.
    pub fn all_effects_applied(&self) -> bool {
        self.enumerate_costs()
            .chain(self.enumerate_rewards())
            .all(|(i, e)| self.applied.contains(&i) || e.effect.is_immediate())
    }
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

    pub fn choice_mut(&mut self, index: NarrativeChoiceIndex) -> &mut NarrativeEventChoice {
        &mut self.choices[index.value]
    }
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
    NarrativeEvent(NarrativeEventData),
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
    /// Stack of interstitial screens the player can view during an adventure.
    pub screens: AdventureScreens,
    /// Deck being used for this adventure
    pub deck: Deck,
    /// Customization options for this adventure
    pub config: AdventureConfiguration,
}

/// Specifies the parameters for picking a card from a set
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CardSelector {
    /// Minimum rarity for cards. Defaults to common.
    pub minimum_rarity: Rarity,
    /// Card types to select from.
    ///
    /// If empty, all card types are allowed.
    pub card_types: EnumSet<CardType>,
    /// Card subtypes to select from.
    ///
    /// If empty, all card subtypes are allowed.    
    pub card_subtypes: EnumSet<CardSubtype>,
    /// True if only upgraded cards should be matched
    pub upgraded: bool,
}

impl Default for CardSelector {
    fn default() -> Self {
        Self {
            minimum_rarity: Rarity::Common,
            card_types: EnumSet::new(),
            card_subtypes: EnumSet::new(),
            upgraded: false,
        }
    }
}

impl CardSelector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rarity(mut self, rarity: Rarity) -> Self {
        self.minimum_rarity = rarity;
        self
    }

    pub fn card_type(mut self, card_type: CardType) -> Self {
        self.card_types.insert(card_type);
        self
    }

    pub fn card_subtype(mut self, card_subtype: CardSubtype) -> Self {
        self.card_subtypes.insert(card_subtype);
        self
    }

    pub fn upgraded(mut self, upgraded: bool) -> Self {
        self.upgraded = upgraded;
        self
    }
}
