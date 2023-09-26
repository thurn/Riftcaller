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

//! Functions  for providing AI responses to the user

use std::time::Instant;

use ai_core::agent::{Agent, AgentConfig};
use ai_game_integration::agents;
use ai_game_integration::state_node::SpelldawnState;
use anyhow::Result;
use database::Database;
use display::render;
use game_data::game_actions::GameAction;
use game_data::game_state::GameState;
use game_data::game_updates::{UpdateState, UpdateTracker};
use game_data::player_name::PlayerId;
use game_data::primitives::{GameId, Milliseconds, Side};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::DelayCommand;
use rules::flags;
use tracing::{debug, info, info_span, subscriber, Instrument, Level};
use tutorial::tutorial_actions;
use with_error::{fail, WithError};

use crate::game_server;
use crate::server_data::{ClientData, GameResponse, RequestData};

/// Whether incremental updates should be sent to the connected player during
/// AI turns.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IncrementalUpdates {
    Send,
    Skip,
}

/// Respond to the current player by spawning a thread to produce an AI
/// response, if an AI is configured for this game and it is currently their
/// turn to act. Returns true if an AI was invoked.
pub async fn maybe_run_ai(
    data: &RequestData,
    game: &mut GameState,
    send_updates: IncrementalUpdates,
) -> Result<bool> {
    if active_agent(game).is_none() {
        return Ok(false);
    };

    let player_id = data.player_id;
    let client_data = ClientData::propagate(data);
    let span = info_span!(">>> run_agent_loop", ?player_id, ?client_data.game_id);
    let future = run_agent_loop(player_id, client_data, send_updates, game).instrument(span);
    if let Err(e) = future.await {
        fail!("Error running agent {:?}", e);
    }
    Ok(true)
}

/// Manually runs the agent-response loop, for use in tests. Do not call in
/// production.
pub async fn run_agent_loop_for_tests(
    database: &impl Database,
    game_id: GameId,
    player_id: PlayerId,
) -> Result<()> {
    let mut game =
        database.fetch_game(game_id).await?.with_error(|| format!("Game not found {game_id}"))?;
    run_agent_loop(
        player_id,
        ClientData { game_id: Some(game_id), adventure_id: None },
        IncrementalUpdates::Skip,
        &mut game,
    )
    .await
}

async fn run_agent_loop(
    player_id: PlayerId,
    context: ClientData,
    send_updates: IncrementalUpdates,
    game: &mut GameState,
) -> Result<()> {
    let mut last_step_time = None;

    loop {
        let Some((side, agent)) = active_agent(game) else {
            break;
        };

        send_snapshot_to_player(player_id, context, send_updates, last_step_time, game).await?;

        let agent_name = agent.name();
        info!(?agent_name, ?player_id, ?game.id, "Picking agent action");
        let action = pick_action(&SpelldawnState(game.clone()), agent).await?;
        {
            let _span = info_span!("apply_agent_action", ?action, ?player_id, ?game.id).entered();
            info!(?action, ?player_id, ?game.id, "Got agent action");
            game.updates = UpdateTracker::new(UpdateState::Push);
            game_server::apply_game_action(game, side, &action)?;
        };
        last_step_time = Some(Instant::now());
    }

    send_snapshot_to_player(player_id, context, send_updates, last_step_time, game).await?;

    Ok(())
}

async fn send_snapshot_to_player(
    player_id: PlayerId,
    context: ClientData,
    send_updates: IncrementalUpdates,
    last_step: Option<Instant>,
    game: &GameState,
) -> Result<()> {
    if send_updates == IncrementalUpdates::Skip {
        return Ok(());
    }

    let rendered = render::render_updates(game, game.player_side(player_id)?)?;

    // Insert a minimum delay to make actions understandable
    let mut response = GameResponse::new(context);
    if let Some(last_step_time) = last_step {
        let elapsed = Instant::now().duration_since(last_step_time).as_millis() as u32;
        if elapsed < 1000 {
            response = response.command(Command::Delay(DelayCommand {
                duration: Some(adapters::time_value(Milliseconds(1000 - elapsed))),
            }));
        }
    }
    let commands = response.commands(rendered).build().user_response;
    debug!(?player_id, ?game.id, "Sending incremental AI response to player");
    crate::send_player_response(Some((player_id, commands))).await;
    Ok(())
}

/// Returns a ([Side], [Agent]) tuple for an agent that can currently act in
/// this game, if one exists.
fn active_agent(game: &GameState) -> Option<(Side, Box<dyn Agent<SpelldawnState>>)> {
    for side in enum_iterator::all::<Side>() {
        if let PlayerId::AI(name) = game.player(side).id {
            if flags::has_priority(game, side) {
                let agent = agents::get(name);
                if !agent.inactive() {
                    return Some((side, agent));
                }
            }
        }
    }
    None
}

async fn pick_action(
    game: &SpelldawnState,
    agent: Box<dyn Agent<SpelldawnState>>,
) -> Result<GameAction> {
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::WARN).finish();
    subscriber::with_default(error_subscriber, || {
        if game.info.config.scripted_tutorial {
            tutorial_actions::current_opponent_action(game)
        } else {
            agent.pick_action(AgentConfig::with_deadline(3), game)
        }
    })
}
