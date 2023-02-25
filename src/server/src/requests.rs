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
use core_ui::prelude::*;
use database::Database;
use game_data::game::GameState;
use game_data::player_name::PlayerId;
use game_data::primitives::GameId;
use game_data::updates::{UpdateTracker, Updates};
use panel_address::PanelAddress;
use player_data::PlayerData;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    LoadSceneCommand, RenderScreenOverlayCommand, SceneLoadMode, UpdatePanelsCommand,
};
use routing::all_panels;
use rules::dispatch;
use screen_overlay::ScreenOverlay;
use with_error::WithError;

use crate::requests;
use crate::server_data::{GameResponse, RequestData};

/// Fetches the current state of the current game from the database, applies a
/// mutation function to it, and then writes the result back to the database.
pub async fn with_game(
    database: &impl Database,
    data: &RequestData,
    mut fun: impl FnMut(&mut GameState) -> Result<GameResponse>,
) -> Result<GameResponse> {
    let mut game = fetch_game(database, data.game_id).await?;
    let mut result = fun(&mut game)?;
    add_panels(database, data.player_id, None, &mut result).await?;
    database.write_game(&game).await?;
    Ok(result)
}

/// Fetches the current state of the current player from the database, applies a
/// mutation function to it, and then writes the result back to the database.
pub async fn with_player(
    database: &impl Database,
    data: &RequestData,
    mut fun: impl FnMut(&mut PlayerData) -> Result<GameResponse>,
) -> Result<GameResponse> {
    let mut player = fetch_player(database, data.player_id).await?;
    let mut result = fun(&mut player)?;
    add_panels(database, data.player_id, Some(&player), &mut result).await?;
    result.push_command(requests::update_screen_overlay(&player));
    database.write_player(&player).await?;
    Ok(result)
}

/// Looks up a player by ID in the database. Prefer to use [with_player] if
/// possible.
pub async fn fetch_player(database: &impl Database, player_id: PlayerId) -> Result<PlayerData> {
    database.fetch_player(player_id).await?.with_error(|| format!("Player not found {player_id}"))
}

/// Looks up a player by ID in the database. Prefer to use [with_game] if
/// possible.
pub async fn fetch_game(database: &impl Database, game_id: Option<GameId>) -> Result<GameState> {
    let id = game_id.with_error(|| "Expected GameId to be included with client request")?;
    let mut game = database.fetch_game(id).await?.with_error(|| format!("Game not found {id}"))?;
    dispatch::populate_delegate_cache(&mut game);
    game.updates = UpdateTracker::new(if game.data.config.simulation {
        Updates::Ignore
    } else {
        Updates::Push
    });
    Ok(game)
}

/// Builds a command to update the screen overlay (UI chrome)
pub fn update_screen_overlay(player: &PlayerData) -> Command {
    Command::RenderScreenOverlay(RenderScreenOverlayCommand {
        node: ScreenOverlay::new(player).build(),
    })
}

/// Requests to switch to a new scene if it's not currently being displayed
pub fn load_scene(name: impl Into<String>) -> Command {
    Command::LoadScene(LoadSceneCommand {
        scene_name: name.into(),
        mode: SceneLoadMode::Single.into(),
        skip_if_current: true,
    })
}

pub fn force_load_scene(name: impl Into<String>) -> Command {
    Command::LoadScene(LoadSceneCommand {
        scene_name: name.into(),
        mode: SceneLoadMode::Single.into(),
        skip_if_current: false,
    })
}

pub async fn add_panels(
    database: &impl Database,
    player_id: PlayerId,
    player: Option<&PlayerData>,
    response: &mut GameResponse,
) -> Result<()> {
    // Currently we unconditionally include all panels with every response. This
    // is something to revisit in the future if the payload size becomes too large.
    let panels = if let Some(p) = player {
        all_panels::player_panels(p)
            .into_iter()
            .map(PanelAddress::PlayerPanel)
            .chain(all_panels::standard_panels().into_iter().map(PanelAddress::StandardPanel))
            .collect::<Vec<PanelAddress>>()
    } else {
        all_panels::standard_panels()
            .into_iter()
            .map(PanelAddress::StandardPanel)
            .collect::<Vec<PanelAddress>>()
    };
    if let Some(command) = fetch_panels(database, player_id, player, &panels).await? {
        response.insert_command(0, command);
    }
    Ok(())
}

/// Fetches the rendered version of the panels provided in the `panels` slice.
pub async fn fetch_panels(
    database: &impl Database,
    player_id: PlayerId,
    player: Option<&PlayerData>,
    panels: &[PanelAddress],
) -> Result<Option<Command>> {
    if panels.is_empty() {
        return Ok(None);
    }

    let mut standard_panels = vec![];
    let mut player_panels = vec![];
    for panel in panels {
        match panel {
            PanelAddress::StandardPanel(p) => standard_panels.push(p),
            PanelAddress::PlayerPanel(p) => player_panels.push(p),
        }
    }

    let mut panels = vec![];
    for panel in standard_panels {
        if let Some(p) = routing::render_standard_panel(*panel)? {
            panels.push(p);
        }
    }

    if !player_panels.is_empty() {
        fetch_player_if_needed(database, player_id, player, |player_data| {
            for panel in &player_panels {
                if let Some(p) = routing::render_player_panel(player_data, **panel)? {
                    panels.push(p);
                }
            }
            Ok(())
        })
        .await?;
    }

    Ok(Some(Command::UpdatePanels(UpdatePanelsCommand { panels })))
}

/// Fetches information for the current player if it is not already populated in
/// `player`.
async fn fetch_player_if_needed(
    database: &impl Database,
    player_id: PlayerId,
    player: Option<&PlayerData>,
    mut fun: impl FnMut(&PlayerData) -> Result<()>,
) -> Result<()> {
    match player {
        Some(p) => fun(p),
        None => {
            let player = fetch_player(database, player_id).await?;
            fun(&player)
        }
    }
}
