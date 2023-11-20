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
use core_ui::panels::Panels;
use database::Database;
use panel_address::StandardPanel;
use player_data::PlayerState;
use tracing::info;

use crate::requests::{self, SceneName};
use crate::server_data::{ClientData, GameResponse};

pub async fn connect(_: &impl Database, player: &PlayerState) -> Result<GameResponse> {
    info!(?player.id, "Connected");
    let mut commands = vec![requests::load_scene(SceneName::Main)];
    commands.push(Panels::open(StandardPanel::MainMenu).into());
    let client_data = ClientData { adventure_id: None, game_id: None };
    let mut result = GameResponse::new(client_data).commands(commands);
    requests::add_standard_ui(&mut result, player, None, None).await?;
    Ok(result)
}
