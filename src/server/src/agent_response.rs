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

use std::sync::atomic::{AtomicBool, Ordering};

use ai_core::agent::{Agent, AgentConfig};
use ai_game_integration::agents;
use ai_game_integration::state_node::SpelldawnState;
use anyhow::Result;
use concurrent_queue::ConcurrentQueue;
use database::Database;
use game_data::game::GameState;
use game_data::game_actions::GameAction;
use game_data::player_name::{NamedPlayer, PlayerId};
use game_data::primitives::{GameId, Side};
use once_cell::sync::Lazy;
use protos::spelldawn::{CommandList, PlayerIdentifier};
use tracing::{info, info_span, subscriber, warn, Level};
use tutorial::tutorial_actions;
use with_error::fail;

use crate::requests;

// This feels safe-ish?
static AGENT_RUNNING: AtomicBool = AtomicBool::new(false);

/// Queue of agent responses that need to be sent to the client, used in offline
/// mode
pub static RESPONSES: Lazy<ConcurrentQueue<CommandList>> = Lazy::new(ConcurrentQueue::unbounded);

/// What to do with responses produced by the agent.
pub enum HandleRequest {
    /// Send each response to the the player who initiated the `GameRequest`.
    SendToPlayer,

    /// Store each response in the [RESPONSES] queue for use by the plugin.
    PushQueue,
}

/// Respond to a player by producing an AI response, if any AI agents are
/// connected.
pub fn handle_request_if_active(
    mut database: impl Database + 'static,
    player_id: Option<&PlayerIdentifier>,
    handle_request: HandleRequest,
) -> Result<()> {
    let respond_to = requests::player_id(&mut database, player_id)?;
    let game_id = match player_data::current_game_id(database.player(respond_to)?) {
        Some(game_id) => game_id,
        _ => return Ok(()),
    };
    let game = database.game(game_id)?;

    if active_agent(&game).is_some() && !AGENT_RUNNING.swap(true, Ordering::Relaxed) {
        info!(?player_id, ?game_id, "Computing agent response");
        tokio::spawn(async move {
            run_agent_loop(database, game_id, respond_to, handle_request)
                .await
                .expect("Error running agent");
            AGENT_RUNNING.store(false, Ordering::Relaxed);
        });
    }
    Ok(())
}

/// Returns a ([Side], [AgentData]) tuple for an agent that can currently act in
/// this game, if one exists.
fn active_agent(game: &GameState) -> Option<(Side, Box<dyn Agent<SpelldawnState>>)> {
    for side in enum_iterator::all::<Side>() {
        if let PlayerId::Named(name) = game.player(side).id {
            if name != NamedPlayer::NoAction && actions::can_take_action(game, side) {
                return Some((side, agents::get(name)));
            }
        }
    }
    None
}

async fn run_agent_loop(
    mut database: impl Database,
    game_id: GameId,
    respond_to: PlayerId,
    handle_request: HandleRequest,
) -> Result<()> {
    loop {
        let game = SpelldawnState(database.game(game_id)?);
        let commands = if let Some((side, agent)) = active_agent(&game) {
            let _span = info_span!("pick_agent_action", ?respond_to, ?game_id).entered();
            info!(?game_id, ?respond_to, "Picking agent action");
            let action = pick_action(&game, agent)?;

            warn!(?action, ?respond_to, ?game_id, "Agent Action");
            let response = requests::handle_game_action(
                &mut database,
                game.player(side).id,
                Some(game_id),
                action,
            )?;

            match response.opponent_response {
                Some((oid, response)) if oid == respond_to => response,
                _ if game.player(side).id == respond_to => response.command_list,
                _ => {
                    fail!("Unknown PlayerId {:?}", respond_to);
                }
            }
        } else {
            return Ok(());
        };

        match handle_request {
            HandleRequest::SendToPlayer => {
                requests::send_player_response(Some((respond_to, commands))).await;
            }
            HandleRequest::PushQueue => {
                RESPONSES.push(commands)?;
            }
        }
    }
}

fn pick_action(game: &SpelldawnState, agent: Box<dyn Agent<SpelldawnState>>) -> Result<GameAction> {
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::WARN).finish();
    subscriber::with_default(error_subscriber, || {
        if game.data.config.scripted_tutorial {
            tutorial_actions::current_opponent_action(game)
        } else {
            agent.pick_action(AgentConfig::with_deadline(3), game)
        }
    })
}
