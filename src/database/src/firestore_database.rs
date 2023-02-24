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
use async_trait::async_trait;
use firestore::FirestoreDb;
use game_data::game::GameState;
use game_data::player_name::PlayerId;
use game_data::primitives::GameId;
use player_data::PlayerData;
use with_error::WithError;

use crate::Database;

pub struct FirestoreDatabase {
    db: FirestoreDb,
}

impl FirestoreDatabase {
    pub async fn new(project_id: impl Into<String>) -> Result<Self> {
        Ok(Self { db: FirestoreDb::new(project_id.into()).await? })
    }
}

#[async_trait]
impl Database for FirestoreDatabase {
    async fn fetch_player(&self, id: PlayerId) -> Result<Option<PlayerData>> {
        self.db
            .fluent()
            .select()
            .by_id_in("players")
            .obj()
            .one(id.to_string())
            .await
            .with_error(|| format!("Error fetching player {id}"))
    }

    async fn write_player(&self, player: &PlayerData) -> Result<()> {
        self.db
            .fluent()
            .insert()
            .into("players")
            .document_id(player.id.to_string())
            .object(player)
            .execute()
            .await?;
        Ok(())
    }

    async fn fetch_game(&self, id: GameId) -> Result<Option<GameState>> {
        self.db
            .fluent()
            .select()
            .by_id_in("games")
            .obj()
            .one(id.to_string())
            .await
            .with_error(|| format!("Error fetching game {id}"))
    }

    async fn write_game(&self, game: &GameState) -> Result<()> {
        self.db
            .fluent()
            .insert()
            .into("games")
            .document_id(game.id.to_string())
            .object(game)
            .execute()
            .await?;
        Ok(())
    }
}
