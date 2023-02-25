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
use game_data::game::GameState;
use game_data::player_name::PlayerId;
use game_data::primitives::GameId;
use once_cell::sync::Lazy;
use player_data::PlayerData;
use serde_json::{de, ser};
use sled::{Db, Tree};
use with_error::{fail, WithError};

use crate::Database;

static DATABASE: Lazy<Db> = Lazy::new(|| sled::open("db").expect("Unable to open database"));

pub struct SledDatabase;

#[async_trait]
impl Database for SledDatabase {
    async fn fetch_player(&self, id: PlayerId) -> Result<Option<PlayerData>> {
        players()?
            .get(player_id_key(id)?)
            .with_error(|| format!("Error fetching player {id}"))?
            .map(|slice| {
                de::from_slice::<PlayerData>(&slice)
                    .with_error(|| format!("Error deserializing player {id}"))
            })
            .transpose()
    }

    async fn write_player(&self, player: &PlayerData) -> Result<()> {
        players()?.insert(
            player_id_key(player.id)?,
            ser::to_vec(player).with_error(|| format!("Error serializing player {}", player.id))?,
        )?;
        Ok(())
    }

    async fn fetch_game(&self, id: GameId) -> Result<Option<GameState>> {
        games()?
            .get(game_id_key(id))
            .with_error(|| format!("Error fetching game {id}"))?
            .map(|slice| {
                de::from_slice::<GameState>(&slice)
                    .with_error(|| format!("Error deserializing game {id}"))
            })
            .transpose()
    }

    async fn write_game(&self, game: &GameState) -> Result<()> {
        games()?.insert(
            game_id_key(game.id),
            ser::to_vec(game).with_error(|| format!("Error serializing game {}", game.id))?,
        )?;
        Ok(())
    }
}

fn games() -> Result<Tree> {
    DATABASE.open_tree("games").with_error(|| "Error opening the 'games' tree")
}

fn players() -> Result<Tree> {
    DATABASE.open_tree("players").with_error(|| "Error opening the 'players' tree")
}

fn player_id_key(player_id: PlayerId) -> Result<[u8; 16]> {
    match player_id {
        PlayerId::Database(key) => Ok(key.0.to_be_bytes()),
        _ => fail!("Expected PlayerId::Database"),
    }
}

fn game_id_key(game_id: GameId) -> [u8; 16] {
    game_id.as_u128().to_be_bytes()
}