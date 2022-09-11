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
use crate::player_name::NamedPlayer;
use crate::primitives::{ActionCount, DeckId, ManaValue, PointsValue, Side};

/// Actions that can be taken from the debug panel, should not be exposed in
/// production.
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum DebugAction {
    // Creates a new game with ID 0, using the canonical decklist for [Side], playing against an
    // opponent who will take no actions. Overwrites the current player's player data with the
    // canonical decklists.
    NewGame(Side),

    // Adds the current player to the game with ID 0, overwriting the non-human player in this
    // game. Overwrites the current player's player data with the canonical decklists.
    JoinGame,

    // Swaps which side the current player is playing as in their current game.
    FlipViewpoint,

    AddMana(ManaValue),
    AddActionPoints(ActionCount),
    AddScore(PointsValue),
    SaveState(u64),
    LoadState(u64),
    SetNamedPlayer(Side, NamedPlayer),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum DeckEditorAction {
    // Add one copy of a card to a deck
    AddToDeck(CardName, DeckId),
    // Remove one copy of a card from a deck
    RemoveFromDeck(CardName, DeckId),
}

/// All possible action payloads that can be sent from a client
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum UserAction {
    GameAction(GameAction),
    Debug(DebugAction),
    DeckEditorAction(DeckEditorAction),
}
