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

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use database::Database;
use game_data::game::GameState;
use game_data::player_name::PlayerId;
use game_data::primitives::GameId;
use player_data::PlayerData;
use protos::spelldawn::PlayerIdentifier;

#[derive(Clone, Debug, Default)]
pub struct FakeDatabase {
    pub generated_game_id: Option<GameId>,
    pub game: Option<GameState>,
    pub players: HashMap<PlayerId, PlayerData>,
}

impl FakeDatabase {
    pub fn game(&self) -> &GameState {
        self.game.as_ref().expect("game")
    }

    pub fn game_mut(&mut self) -> &mut GameState {
        self.game.as_mut().expect("game")
    }
}

#[async_trait]
impl Database for FakeDatabase {
    fn generate_game_id(&self) -> GameId {
        GameId::generate()
    }

    async fn fetch_player(&self, id: PlayerId) -> Result<Option<PlayerData>> {
        Ok(Some(self.players[&id].clone()))
    }

    async fn write_player(&mut self, player: &PlayerData) -> Result<()> {
        self.players.insert(player.id, player.clone());
        Ok(())
    }

    async fn fetch_game(&self, _id: GameId) -> Result<Option<GameState>> {
        Ok(Some(self.game.clone().expect("game")))
    }

    async fn write_game(&mut self, game: &GameState) -> Result<()> {
        self.game = Some(game.clone());
        Ok(())
    }
}

pub fn to_player_identifier(id: PlayerId) -> PlayerIdentifier {
    let value = match id {
        PlayerId::Database(value) => value,
        _ => panic!("Unsupported PlayerId type"),
    };

    PlayerIdentifier { ulid: value.to_string() }
}
