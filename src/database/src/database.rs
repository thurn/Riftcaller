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

//! Core database implementation, handles querying and storing game state.

pub mod sled_database;

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use game_data::game::GameState;
use game_data::player_name::PlayerId;
use game_data::primitives::GameId;
use player_data::PlayerData;

#[async_trait]
pub trait Database: Send + Sync {
    fn generate_game_id(&self) -> GameId {
        GameId::generate()
    }

    async fn fetch_player(&self, id: PlayerId) -> Result<Option<PlayerData>>;

    async fn write_player(&mut self, player: &PlayerData) -> Result<()>;

    async fn fetch_game(&self, id: GameId) -> Result<Option<GameState>>;

    async fn write_game(&mut self, game: &GameState) -> Result<()>;
}

#[derive(Default)]
pub struct InMemoryDatabase {
    players: HashMap<PlayerId, PlayerData>,
    games: HashMap<GameId, GameState>,
}

#[async_trait]
impl Database for InMemoryDatabase {
    async fn fetch_player(&self, id: PlayerId) -> Result<Option<PlayerData>> {
        if self.players.contains_key(&id) {
            Ok(Some(self.players[&id].clone()))
        } else {
            Ok(None)
        }
    }

    async fn write_player(&mut self, player: &PlayerData) -> Result<()> {
        self.players.insert(player.id, player.clone());
        Ok(())
    }

    async fn fetch_game(&self, id: GameId) -> Result<Option<GameState>> {
        if self.games.contains_key(&id) {
            Ok(Some(self.games[&id].clone()))
        } else {
            Ok(None)
        }
    }

    async fn write_game(&mut self, game: &GameState) -> Result<()> {
        self.games.insert(game.id, game.clone());
        Ok(())
    }
}
