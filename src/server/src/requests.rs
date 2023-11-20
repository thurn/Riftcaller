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

use std::fmt::{self, Display};

use anyhow::Result;
use core_ui::prelude::*;
use database::Database;
use display::set_display_preference;
use game_data::animation_tracker::{AnimationState, AnimationTracker};
use game_data::game_actions::DisplayPreference;
use game_data::game_state::GameState;
use game_data::player_name::PlayerId;
use game_data::primitives::GameId;
use panel_address::PanelAddress;
use player_data::PlayerState;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    LoadSceneCommand, RenderScreenOverlayCommand, SceneLoadMode, UpdatePanelsCommand,
};
use routing::all_panels;
use rules::dispatch;
use screen_overlay::ScreenOverlay;
use with_error::WithError;

use crate::keyboard_shortcuts;
use crate::server_data::{GameResponse, RequestData};

/// Fetches the current state of the current game from the database, applies a
/// mutation function to it, and then writes the result back to the database.
pub async fn with_game(
    database: &impl Database,
    data: &RequestData,
    mut fun: impl FnMut(&mut GameState) -> Result<GameResponse>,
) -> Result<GameResponse> {
    // Currently we unconditionally fetch the player every time and include all
    // panels with every response. This is something to revisit in the future if
    // the payload size/database queries become a problem.
    let mut game = fetch_game(database, data.game_id).await?;
    let user_side = game.player_side(data.player_id)?;
    let mut result = fun(&mut game)?;
    let player = fetch_player(database, data.player_id).await?;
    add_standard_ui(
        &mut result,
        &player,
        Some(&game),
        set_display_preference::button(&game, user_side, None),
    )
    .await?;
    database.write_game(&game).await?;
    Ok(result)
}

/// Fetches the current state of the current player from the database, applies a
/// mutation function to it, and then writes the result back to the database.
pub async fn with_player(
    database: &impl Database,
    data: &RequestData,
    mut fun: impl FnMut(&mut PlayerState) -> Result<GameResponse>,
) -> Result<GameResponse> {
    let mut player = fetch_player(database, data.player_id).await?;
    let mut result = fun(&mut player)?;
    add_standard_ui(&mut result, &player, None, None).await?;
    database.write_player(&player).await?;
    Ok(result)
}

/// Looks up a player by ID in the database. Prefer to use [with_player] if
/// possible.
pub async fn fetch_player(database: &impl Database, player_id: PlayerId) -> Result<PlayerState> {
    database.fetch_player(player_id).await?.with_error(|| format!("Player not found {player_id}"))
}

/// Looks up a player by ID in the database. Returns an error if `game_id` is
/// `None`.
pub async fn fetch_game(database: &impl Database, game_id: Option<GameId>) -> Result<GameState> {
    let id = game_id.with_error(|| "Expected GameId to be included with client request")?;
    let mut game = database.fetch_game(id).await?.with_error(|| format!("Game not found {id}"))?;
    dispatch::populate_delegate_cache(&mut game);
    game.animations = AnimationTracker::new(if game.info.config.simulation {
        AnimationState::Ignore
    } else {
        AnimationState::Track
    });
    Ok(game)
}

pub enum SceneName {
    Game,
    World,
    Main,
}

impl Display for SceneName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SceneName::Game => write!(f, "Game"),
            SceneName::World => write!(f, "World"),
            SceneName::Main => write!(f, "Main"),
        }
    }
}

/// Requests to switch to a new scene if it's not currently being displayed
pub fn load_scene(name: SceneName) -> Command {
    Command::LoadScene(LoadSceneCommand {
        scene_name: name.to_string(),
        mode: SceneLoadMode::Single.into(),
        skip_if_current: true,
    })
}

pub fn force_load_scene(name: SceneName) -> Command {
    Command::LoadScene(LoadSceneCommand {
        scene_name: name.to_string(),
        mode: SceneLoadMode::Single.into(),
        skip_if_current: false,
    })
}

pub async fn add_standard_ui(
    response: &mut GameResponse,
    player: &PlayerState,
    game: Option<&GameState>,
    set_display_preference: Option<DisplayPreference>,
) -> Result<()> {
    response.insert_command(
        0,
        Command::RenderScreenOverlay(RenderScreenOverlayCommand {
            node: ScreenOverlay::new(player)
                .game(game)
                .set_display_preference_button(set_display_preference)
                .build(),
        }),
    );

    response.insert_command(0, keyboard_shortcuts::build(player, game));

    let panels = all_panels::standard_panels()
        .into_iter()
        .map(PanelAddress::StandardPanel)
        .chain(all_panels::player_panels(player).into_iter().map(PanelAddress::PlayerPanel))
        .collect::<Vec<PanelAddress>>();

    if let Some(command) = render_panels(player, &panels).await? {
        response.insert_command(0, command);
    }
    Ok(())
}

/// Fetches the rendered version of the panels provided in the `panels` slice.
pub async fn render_panels(
    player: &PlayerState,
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
        for panel in &player_panels {
            if let Some(p) = routing::render_player_panel(player, **panel)? {
                panels.push(p);
            }
        }
    }

    Ok(Some(Command::UpdatePanels(UpdatePanelsCommand { panels })))
}
