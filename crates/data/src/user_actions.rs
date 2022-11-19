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

use serde::{Deserialize, Serialize};

use crate::card_name::CardName;
use crate::game_actions::GameAction;
use crate::player_name::{NamedPlayer, PlayerId};
use crate::primitives::{ActionCount, DeckIndex, GameId, ManaValue, PointsValue, School, Side};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct NewGameDebugOptions {
    /// If true, all game events will be non-random
    pub deterministic: bool,
    /// Explicitly set the ID for this game
    pub override_game_id: Option<GameId>,
}

/// Action to initiate a new game
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct NewGameAction {
    /// Deck to use for this game
    pub deck_index: DeckIndex,
    /// Opponent to play against
    pub opponent: PlayerId,
    /// Debug configuration for this game
    pub debug_options: Option<NewGameDebugOptions>,
}

/// Actions that can be taken from the debug panel, should not be exposed in
/// production.
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum DebugAction {
    /// Creates a new game with ID 0, using the canonical decklist for [Side],
    /// playing against an opponent who will take no actions. Overwrites the
    /// current player's player data with the canonical decklists.
    NewGame(Side),

    /// Adds the current player to the game with ID 0, overwriting the non-human
    /// player in this game. Overwrites the current player's player data
    /// with the canonical decklists.
    JoinGame,

    /// Swaps which side the current player is playing as in their current game.
    FlipViewpoint,

    AddMana(ManaValue),
    AddActionPoints(ActionCount),
    AddScore(PointsValue),
    SaveState(u64),
    LoadState(u64),
    SetNamedPlayer(Side, NamedPlayer),

    /// Gives the player copies of every card
    FullCollection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum DeckEditorAction {
    /// Create a new deck for the current player
    CreateDeck(Side, School),
    /// Add one copy of a card to a deck
    AddToDeck(CardName, DeckIndex),
    /// Remove one copy of a card from a deck
    RemoveFromDeck(CardName, DeckIndex),
}

impl From<DeckEditorAction> for UserAction {
    fn from(a: DeckEditorAction) -> Self {
        UserAction::DeckEditorAction(a)
    }
}

/// All possible action payloads that can be sent from a client
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum UserAction {
    NewGame(NewGameAction),
    Debug(DebugAction),
    GameAction(GameAction),
    DeckEditorAction(DeckEditorAction),
}
