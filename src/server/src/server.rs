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

pub mod adventure_server;
pub mod ai_agent_response;
pub mod debug_server;
pub mod game_server;
pub mod main_menu_server;
pub mod new_game;
pub mod player_server;
pub mod requests;
pub mod server_data;

use anyhow::Result;
use dashmap::DashMap;
use database::Database;
use game_data::player_name::PlayerId;
use once_cell::sync::Lazy;
use panel_address::PanelAddress;
use player_data::PlayerData;
use protos::spelldawn::client_action::Action;
use protos::spelldawn::spelldawn_server::Spelldawn;
use protos::spelldawn::{
    CommandList, ConnectRequest, FetchPanelAction, GameRequest, PlayerIdentifier, StandardAction,
};
use serde_json::de;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info, info_span, warn, Instrument};
use ulid::Ulid;
use user_action_data::UserAction;
use with_error::WithError;

use crate::server_data::{ClientData, GameResponse, RequestData};

/// Stores active channels for each user.
///
/// TODO: Clean this up on disconnect. This is quite easy to do with 'real' gRPC
/// but I haven't figured out how to do it with gRPC-web (which is just
/// fake-streaming over HTTP1). Unity doesn't support HTTP2 natively, but it's
/// possible to do it via a third party networking stack.
static CHANNELS: Lazy<DashMap<PlayerId, Sender<Result<CommandList, Status>>>> =
    Lazy::new(DashMap::new);

pub struct GameService<T: Database> {
    pub database: T,
}

#[tonic::async_trait]
impl<T: Database + 'static> Spelldawn for GameService<T> {
    type ConnectStream = ReceiverStream<Result<CommandList, Status>>;

    async fn connect(
        &self,
        request: Request<ConnectRequest>,
    ) -> Result<Response<Self::ConnectStream>, Status> {
        let player_id = parse_client_id(request.get_ref().player_id.as_ref())?;
        let (tx, rx) = mpsc::channel(4);
        let result = handle_connect(&self.database, player_id).await;
        match result {
            Ok(response) => {
                let built = response.build();
                send_player_response(built.opponent_response).await;
                if let Err(error) = tx.send(Ok(built.user_response)).await {
                    error!(?player_id, ?error, "Send Error!");
                    return Err(Status::internal(format!("Send Error:{error:#}")));
                }
            }
            Err(error) => {
                error!(?error, "Connect Error!");
                return Err(Status::internal(format!("Connection Error: {error:#}")));
            }
        }

        CHANNELS.insert(player_id, tx);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn perform_action(
        &self,
        request: Request<GameRequest>,
    ) -> Result<Response<CommandList>, Status> {
        let player_id = parse_client_id(request.get_ref().player_id.as_ref())?;
        let result = handle_action(&self.database, player_id, request.get_ref()).await;
        match result {
            Ok(response) => {
                let built = response.build();
                send_player_response(built.opponent_response).await;
                Ok(Response::new(built.user_response))
            }
            Err(error) => {
                error!(?error, "Server Error!");
                Err(Status::internal(format!("Server Error: {error:#}")))
            }
        }
    }
}

pub async fn handle_connect(database: &impl Database, player_id: PlayerId) -> Result<GameResponse> {
    let player = fetch_or_create_player(database, player_id).await?;
    let mut result = match player.current_activity() {
        player_data::PlayerActivity::None => main_menu_server::connect(database, &player).await,
        player_data::PlayerActivity::Adventure(adventure) => {
            adventure_server::connect(database, &player, adventure).await
        }
        player_data::PlayerActivity::PlayingGame(game_id) => {
            game_server::connect(database, &player, game_id).await
        }
    }?;

    result.push_command(requests::update_screen_overlay(&player));
    Ok(result)
}

pub async fn handle_action(
    database: &impl Database,
    player_id: PlayerId,
    request: &GameRequest,
) -> Result<GameResponse> {
    let action = request
        .action
        .as_ref()
        .with_error(|| "Action is required")?
        .action
        .as_ref()
        .with_error(|| "ClientAction is required")?;

    let metadata = ClientData::from_client_metadata(request.metadata.as_ref())?;

    let data =
        RequestData { player_id, game_id: metadata.game_id, adventure_id: metadata.adventure_id };

    let span = info_span!("handle_action", ?metadata, ?player_id);
    match action {
        Action::StandardAction(a) => {
            handle_standard_action(database, &data, a).instrument(span).await
        }
        Action::FetchPanel(a) => handle_fetch_panel(database, &data, a).instrument(span).await,
        Action::GainMana(a) => {
            game_server::handle_gain_mana(database, &data, a).instrument(span).await
        }
        Action::DrawCard(a) => {
            game_server::handle_draw_card(database, &data, a).instrument(span).await
        }
        Action::PlayCard(a) => {
            game_server::handle_play_card(database, &data, a).instrument(span).await
        }
        Action::LevelUpRoom(a) => {
            game_server::handle_level_up_room(database, &data, a).instrument(span).await
        }
        Action::InitiateRaid(a) => {
            game_server::handle_initiate_raid(database, &data, a).instrument(span).await
        }
        Action::SpendActionPoint(a) => {
            game_server::handle_spend_action_point(database, &data, a).instrument(span).await
        }
    }
}

async fn handle_standard_action(
    database: &impl Database,
    data: &RequestData,
    input: &StandardAction,
) -> Result<GameResponse> {
    let action: UserAction =
        de::from_slice(&input.payload).with_error(|| "Failed to deserialize action payload")?;
    info!(?action, ?data.player_id, "Action");
    let span = info_span!("handle_standard_action", ?action, ?data.player_id, ?data.game_id);

    match action {
        UserAction::Debug(a) => {
            debug_server::handle_debug_action(database, data, &a).instrument(span).await
        }
        UserAction::NewAdventure(side) => {
            adventure_server::handle_new_adventure(database, data, side).instrument(span).await
        }
        UserAction::AdventureAction(a) => {
            adventure_server::handle_adventure_action(database, data, &a).instrument(span).await
        }
        UserAction::LeaveAdventure => {
            adventure_server::handle_leave_adventure(database, data).instrument(span).await
        }
        UserAction::NewGame(a) => new_game::create(database, data, &a).instrument(span).await,
        UserAction::GameAction(a) => {
            game_server::handle_game_action(database, data, &a).instrument(span).await
        }
        UserAction::LeaveGame => {
            game_server::handle_leave_game(database, data).instrument(span).await
        }
        UserAction::DeckEditorAction(a) => {
            player_server::handle_deck_editor_action(database, data, &a).instrument(span).await
        }
    }
}

async fn handle_fetch_panel(
    database: &impl Database,
    data: &RequestData,
    action: &FetchPanelAction,
) -> Result<GameResponse> {
    let address: PanelAddress = de::from_slice(
        &action.panel_address.as_ref().with_error(|| "No panel specified")?.serialized,
    )
    .with_error(|| "deserialization failed")?;
    warn!(?address, ?data.player_id, "Fetch Panel");
    Ok(GameResponse::new(ClientData::propagate(data)).command(
        requests::fetch_panels(database, data.player_id, None, &[address])
            .await?
            .with_error(|| "Panels should be nonempty")?,
    ))
}

/// Sends a game response to a given player, if they are connected to the
/// server.
pub async fn send_player_response(response: Option<(PlayerId, CommandList)>) {
    if let Some((player_id, commands)) = response {
        if let Some(channel) = CHANNELS.get(&player_id) {
            if channel.send(Ok(commands)).await.is_err() {
                // This returns SendError if the client is disconnected, which isn't a
                // huge problem. Hopefully they will reconnect again in the future.
                info!(?player_id, "Client is disconnected");
            }
        }
    }
}

/// Sends an error response to a connected player
pub async fn send_player_error(player_id: PlayerId, error: &anyhow::Error) {
    let Some(channel) = CHANNELS.get(&player_id) else { return };
    // Error sending the error! Oh well.
    let _ = channel.send(Err(Status::internal(format!("Error running agent {error}")))).await;
}

fn parse_client_id(player_id: Option<&PlayerIdentifier>) -> Result<PlayerId, Status> {
    let Some(player_id) = player_id else {
        return Err(Status::invalid_argument("Client player_id is required"));
    };

    match Ulid::from_string(&player_id.ulid) {
        Ok(id) => Ok(PlayerId::new(id)),
        Err(e) => Err(Status::invalid_argument(format!("Invalid player_id. {e:?}"))),
    }
}

async fn fetch_or_create_player(
    database: &impl Database,
    player_id: PlayerId,
) -> Result<PlayerData> {
    Ok(if let Some(player) = database.fetch_player(player_id).await? {
        player
    } else {
        let player = PlayerData::new(player_id);
        database.write_player(&player).await?;
        player
    })
}
