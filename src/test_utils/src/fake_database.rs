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
use std::sync::Mutex;

use anyhow::Result;
use async_trait::async_trait;
use database::Database;
use game_data::game::GameState;
use game_data::player_name::PlayerId;
use game_data::primitives::GameId;
use player_data::PlayerState;
use protos::spelldawn::PlayerIdentifier;

#[derive(Debug, Default)]
pub struct FakeDatabase {
    pub generated_game_id: Option<GameId>,
    pub game: Mutex<Option<GameState>>,
    pub players: Mutex<HashMap<PlayerId, PlayerState>>,
}

impl FakeDatabase {
    pub fn game(&self) -> GameState {
        self.game.lock().unwrap().clone().expect("game")
    }

    pub fn mutate_game(&self, mut fun: impl FnMut(&mut GameState)) {
        let mut game = self.game.lock().unwrap();
        fun(game.as_mut().expect("game"))
    }

    pub fn mutate_player(&self, id: PlayerId, mut fun: impl FnMut(&mut PlayerState)) {
        let mut players = self.players.lock().unwrap();
        fun(players.get_mut(&id).expect("game"))
    }
}

#[async_trait]
impl Database for FakeDatabase {
    fn generate_game_id(&self) -> GameId {
        GameId::generate()
    }

    async fn fetch_player(&self, id: PlayerId) -> Result<Option<PlayerState>> {
        Ok(Some(self.players.lock().unwrap()[&id].clone()))
    }

    async fn write_player(&self, player: &PlayerState) -> Result<()> {
        self.players.lock().unwrap().insert(player.id, player.clone());
        Ok(())
    }

    async fn fetch_game(&self, _id: GameId) -> Result<Option<GameState>> {
        Ok(Some(self.game.lock().unwrap().clone().expect("game")))
    }

    async fn write_game(&self, game: &GameState) -> Result<()> {
        let _ = self.game.lock().unwrap().insert(game.clone());
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
