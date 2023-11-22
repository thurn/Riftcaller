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

use adventure_data::adventure::{AdventureConfiguration, AdventureState};
use adventure_data::adventure_action::AdventureAction;
use anyhow::Result;
use core_data::game_primitives::Side;
use database::Database;
use player_data::PlayerState;
use tracing::info;
use with_error::WithError;

use crate::requests::{self, SceneName};
use crate::server_data::{ClientData, GameResponse, RequestData};

pub async fn connect(
    _: &impl Database,
    player: &PlayerState,
    adventure: &AdventureState,
) -> Result<GameResponse> {
    info!(?player.id, ?adventure.id, "Connected to adventure");
    let mut commands = vec![requests::load_scene(SceneName::World)];
    commands.append(&mut adventure_display::render(adventure)?);
    let client_data = ClientData { adventure_id: Some(adventure.id), game_id: None };
    let mut result = GameResponse::new(client_data).commands(commands);
    requests::add_standard_ui(&mut result, player, None, None).await?;
    Ok(result)
}

pub async fn handle_new_adventure(
    database: &impl Database,
    data: &RequestData,
    side: Side,
) -> Result<GameResponse> {
    requests::with_player(database, data, |player| {
        let adventure =
            adventure_generator::new_adventure(AdventureConfiguration::new(player.id, side));
        let id = adventure.id;
        player.adventure = Some(adventure);
        Ok(GameResponse::new(ClientData::with_adventure_id(data, Some(id)))
            .command(requests::load_scene(SceneName::World)))
    })
    .await
}

pub async fn handle_adventure_action(
    database: &impl Database,
    data: &RequestData,
    action: &AdventureAction,
) -> Result<GameResponse> {
    update_adventure(database, data, |adventure| {
        adventure_actions::handle_adventure_action(adventure, action)
    })
    .await
}

pub async fn handle_leave_adventure(
    database: &impl Database,
    data: &RequestData,
) -> Result<GameResponse> {
    requests::with_player(database, data, |player| {
        player.adventure = None;
        Ok(GameResponse::new(ClientData::with_adventure_id(data, None))
            .command(requests::load_scene(SceneName::Main)))
    })
    .await
}

pub async fn update_adventure(
    database: &impl Database,
    data: &RequestData,
    function: impl Fn(&mut AdventureState) -> Result<()>,
) -> Result<GameResponse> {
    requests::with_player(database, data, |player| {
        let adventure_state =
            player.adventure.as_mut().with_error(|| "Expected active adventure")?;
        let id = adventure_state.id;
        function(adventure_state)?;
        let commands = adventure_display::render(adventure_state)?;
        Ok(GameResponse::new(ClientData::with_adventure_id(data, Some(id))).commands(commands))
    })
    .await
}
