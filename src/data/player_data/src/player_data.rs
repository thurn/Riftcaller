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

use anyhow::Result;
use game_data::adventure::AdventureState;
use game_data::deck::Deck;
use game_data::player_name::PlayerId;
use game_data::primitives::{DeckId, GameId};
use game_data::tutorial_data::TutorialData;
use game_data::user_actions::NewGameAction;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use with_error::WithError;

/// Represents the state of a game the player is participating in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerStatus {
    /// The player has initiated a request to create a game
    RequestedGame(NewGameAction),
    /// The player is currently playing in the [GameId] game.
    Playing(GameId),
}

/// Represents a player's stored data.
///
/// For a player's state *within a given game* see `PlayerState`.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    /// Unique identifier for this player
    pub id: PlayerId,
    /// Identifies the game this player is currently participating in, if any.
    pub status: Option<PlayerStatus>,
    /// State for an ongoing adventure, if any
    pub adventure: Option<AdventureState>,
    /// Data related to this player's tutorial progress
    pub tutorial: TutorialData,
}

impl PlayerData {
    pub fn new(id: PlayerId) -> Self {
        Self { id, status: None, adventure: None, tutorial: TutorialData::default() }
    }

    /// Returns the active [AdventureState] when one is expected to exist
    pub fn adventure(&self) -> Result<&AdventureState> {
        self.adventure.as_ref().with_error(|| "Expected active adventure")
    }

    /// Mutable equivalent of [Self::adventure]
    pub fn adventure_mut(&mut self) -> Result<&mut AdventureState> {
        self.adventure.as_mut().with_error(|| "Expected active adventure")
    }

    /// Retrieves one of a player's decks based on its [DeckId].
    pub fn deck(&self, deck_id: DeckId) -> Result<&Deck> {
        Ok(match deck_id {
            DeckId::Adventure => &self.adventure()?.deck,
        })
    }

    /// Mutable version of [Self::deck]
    pub fn deck_mut(&mut self, deck_id: DeckId) -> Result<&mut Deck> {
        Ok(match deck_id {
            DeckId::Adventure => &mut self.adventure_mut()?.deck,
        })
    }
}

/// Returns the [GameId] an optional [PlayerData] is currently playing in, if
/// any.
pub fn current_game_id(data: Option<PlayerData>) -> Option<GameId> {
    match data.as_ref().and_then(|player| player.status.as_ref()) {
        Some(PlayerStatus::Playing(id)) => Some(*id),
        _ => None,
    }
}
