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

use std::fmt;

use adventure_data::adventure::Coins;
use adventure_data::adventure_action::AdventureAction;
use convert_case::{Case, Casing};
use enum_iterator::Sequence;
use game_data::card_name::{CardMetadata, CardVariant};
use game_data::card_state::CardPosition;
use game_data::game_actions::GameAction;
use game_data::player_name::{AIPlayer, PlayerId};
use game_data::primitives::{ActionCount, DeckId, GameId, ManaValue, PointsValue, Side};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct NewGameDebugOptions {
    /// If true, all game events will be non-random
    pub deterministic: bool,
    /// Explicitly set the ID for this game
    pub override_game_id: Option<GameId>,
}

/// Canonical decklists which can be used in new games
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum NamedDeck {
    EmptyChampion,
    EmptyOverlord,
    ChampionTestSpells,
    OverlordTestSpells,
    BasicChampion,
    TutorialChampion,
    BasicOverlord,
    TutorialOverlord,
    CanonicalChampion,
    CanonicalOverlord,
}

/// Identifies deck to be used in a new game
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum NewGameDeck {
    DeckId(DeckId),
    NamedDeck(NamedDeck),
}

/// Action to initiate a new game
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct NewGameAction {
    /// Deck to use for this game
    pub deck: NewGameDeck,
    /// Opponent to play against
    pub opponent: PlayerId,
    /// Whether to display the new player tutorial
    pub tutorial: bool,
    /// Debug configuration for this game
    pub debug_options: Option<NewGameDebugOptions>,
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize, Sequence, Display)]
pub enum DebugScenario {
    NewGameOverlord,
    NewGameChampion,
    VsInfernalMinionAndScheme,
}

impl DebugScenario {
    pub fn side(&self) -> Side {
        match self {
            DebugScenario::NewGameOverlord => Side::Overlord,
            DebugScenario::NewGameChampion => Side::Champion,
            DebugScenario::VsInfernalMinionAndScheme => Side::Champion,
        }
    }

    pub fn displayed_name(&self) -> String {
        format!("{self}").from_case(Case::Pascal).to_case(Case::Title)
    }
}

/// Actions that can be taken from the debug panel, should not be exposed in
/// production.
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum DebugAction {
    /// Creates a new game with ID 0, using the canonical decklist for [Side],
    /// playing against an opponent who will take no actions. Overwrites the
    /// current player's player data with the canonical decklists.
    NewGame(Side),
    /// Adds the current player to the game with ID 0 with the given side.
    JoinGame(Side),
    /// Swaps which side the current player is playing as in their current game.
    FlipViewpoint,
    AddMana(ManaValue),
    AddActionPoints(ActionCount),
    AddScore(PointsValue),
    SaveGameState(u128),
    LoadGameState(u128),
    SetNamedPlayer(Side, AIPlayer),
    SavePlayerState(u128),
    LoadPlayerState(u128),
    AddCoins(Coins),
    /// Reads text in the card list search field and shows matching cards
    FilterCardList(CardPosition, CardMetadata),
    AddToZone(CardVariant, CardPosition),
    ApplyScenario(DebugScenario),
    DebugUndo,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum DeckEditorAction {
    /// Mark the user as having seen the prompt screen
    ViewedPrompt,
    /// Add one copy of a card to a deck
    AddToDeck(CardVariant),
    /// Remove one copy of a card from a deck
    RemoveFromDeck(CardVariant),
}

impl From<DeckEditorAction> for UserAction {
    fn from(a: DeckEditorAction) -> Self {
        UserAction::DeckEditorAction(a)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum GameOutcome {
    Victory,
    Defeat,
}

/// All possible action payloads that can be sent from a client
#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum UserAction {
    /// Developer actions for debugging
    Debug(DebugAction),

    /// Initiate a new adventure, generating a new map and replacing any
    /// existing adventure.
    NewAdventure(Side),
    /// Perform an action within an ongoing adventure
    AdventureAction(AdventureAction),
    /// Remove a player's current adventure, i.e. to stop displaying the
    /// adventure summary screen. Typically happens *after* the 'abandon
    /// adventure' action transitions it to its summary state.
    LeaveAdventure,

    /// Create a new game (match between two players)
    NewGame(NewGameAction),
    /// Perform an action within a game.
    GameAction(GameAction),

    /// Leave the game that the player is currently playing in. Typically
    /// invoked from the game over screen, the 'resign' action is used to
    /// end the game itself.
    LeaveGame(GameOutcome),

    /// Perform an action in the deck editor
    DeckEditorAction(DeckEditorAction),
}

impl From<GameAction> for UserAction {
    fn from(a: GameAction) -> Self {
        UserAction::GameAction(a)
    }
}

impl From<AdventureAction> for UserAction {
    fn from(a: AdventureAction) -> Self {
        UserAction::AdventureAction(a)
    }
}

impl fmt::Debug for UserAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Debug(a) => f.debug_tuple("Debug").field(a).finish(),
            Self::NewAdventure(a) => f.debug_tuple("NewAdventure").field(a).finish(),
            Self::AdventureAction(a) => write!(f, "{a:?}"),
            Self::LeaveAdventure => write!(f, "LeaveAdventure"),
            Self::NewGame(a) => f.debug_tuple("NewGame").field(a).finish(),
            Self::GameAction(a) => write!(f, "{a:?}"),
            Self::LeaveGame(a) => f.debug_tuple("LeaveGame").field(a).finish(),
            Self::DeckEditorAction(a) => write!(f, "{a:?}"),
        }
    }
}
