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

use adventure_data::adventure::{AdventureScreen, AdventureState, BattleData};
use anyhow::Result;
use core_data::game_primitives::{DeckId, GameId, Side};
use enum_kinds::EnumKind;
use game_data::deck::Deck;
use game_data::player_name::PlayerId;
use game_data::tutorial_data::TutorialData;
use serde::{Deserialize, Serialize};
use user_action_data::NewGameAction;
use with_error::{fail, WithError};

/// Represents the state of a game the player is participating in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerStatus {
    /// The player has initiated a request to create a game
    RequestedGame(NewGameAction),
    /// The player is currently playing in the [GameId] game as the [Side]
    /// player.
    Playing(GameId, Side),
}

/// Identifies the current major activity this player is doing in the game.
#[derive(EnumKind)]
#[enum_kind(PlayerActivityKind, derive(Serialize, Deserialize, Hash))]
pub enum PlayerActivity<'a> {
    None,
    Adventure(&'a AdventureState),
    PlayingGame(GameId, Side),
}

impl<'a> PlayerActivity<'a> {
    pub fn kind(&self) -> PlayerActivityKind {
        self.into()
    }

    pub fn side(&self) -> Option<Side> {
        match self {
            Self::PlayingGame(_, side) => Some(*side),
            _ => None,
        }
    }
}

/// Represents a player's stored data.
///
/// For a player's state *within a given game* see `GamePlayerData`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    /// Unique identifier for this player
    pub id: PlayerId,
    /// Identifies the game this player is currently participating in, if any.
    pub status: Option<PlayerStatus>,
    /// State for an ongoing adventure, if any
    pub adventure: Option<AdventureState>,
    /// Data related to this player's tutorial progress
    pub tutorial: TutorialData,
}

impl PlayerState {
    pub fn new(id: PlayerId) -> Self {
        Self { id, status: None, adventure: None, tutorial: TutorialData::default() }
    }

    /// Returns what this player is currently doing within the game.
    pub fn current_activity(&self) -> PlayerActivity {
        if let Some(PlayerStatus::Playing(game_id, side)) = self.status {
            return PlayerActivity::PlayingGame(game_id, side);
        }

        if let Some(adventure) = &self.adventure {
            return PlayerActivity::Adventure(adventure);
        }

        PlayerActivity::None
    }

    /// Returns the current game this player is playing in, or an error if there
    /// is no such game.
    pub fn current_game_id(&self) -> Result<GameId> {
        if let Some(PlayerStatus::Playing(game_id, _)) = self.status {
            Ok(game_id)
        } else {
            fail!("Player {} is not currently playing in a game", self.id)
        }
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

/// Returns the [GameId] an optional [PlayerState] is currently playing in, if
/// any.
pub fn current_game_id(data: Option<PlayerState>) -> Option<GameId> {
    match data.as_ref().and_then(|player| player.status.as_ref()) {
        Some(PlayerStatus::Playing(id, _)) => Some(*id),
        _ => None,
    }
}

/// Returns the battle tile the player is currently visiting, or None if
/// they are not currently visiting a battle tile.
pub fn current_battle(player: &PlayerState) -> Option<&BattleData> {
    if let AdventureScreen::Battle(data) = player.adventure.as_ref()?.screens.current()? {
        Some(data)
    } else {
        None
    }
}
