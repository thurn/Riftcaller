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

//! Core database implementation, handles querying and storing game state.

use anyhow::Result;
use async_trait::async_trait;
use core_data::game_primitives::GameId;
use game_data::game_state::GameState;
use game_data::player_name::PlayerId;
use player_data::PlayerState;

pub mod firestore_database;
pub mod sled_database;

#[async_trait]
pub trait Database: Send + Sync {
    fn generate_game_id(&self) -> GameId {
        GameId::generate()
    }

    async fn fetch_player(&self, id: PlayerId) -> Result<Option<PlayerState>>;

    async fn write_player(&self, player: &PlayerState) -> Result<()>;

    async fn fetch_game(&self, id: GameId) -> Result<Option<GameState>>;

    async fn write_game(&self, game: &GameState) -> Result<()>;
}
