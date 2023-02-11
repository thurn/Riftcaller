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

//! Top-level server request handling

use actions;
use adapters::ServerCardId;
use adventure_data::adventure::{AdventureConfiguration, AdventureState};
use anyhow::Result;
use core_ui::panels::Panels;
use core_ui::prelude::Component;
use dashmap::DashMap;
use database::{Database, SledDatabase};
use deck_editor::deck_editor_actions;
use display::render;
use game_data::deck::Deck;
use game_data::game::{GameConfiguration, GameState};
use game_data::game_actions;
use game_data::game_actions::GameAction;
use game_data::player_name::{NamedPlayer, PlayerId};
use game_data::primitives::{GameId, ResponseContext, Side};
use game_data::tutorial_data::TutorialData;
use game_data::updates::{UpdateTracker, Updates};
use once_cell::sync::Lazy;
use panel_address::PanelAddress;
use player_data::{PlayerData, PlayerStatus};
use protos::spelldawn::client_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::spelldawn_server::Spelldawn;
use protos::spelldawn::{
    card_target, CardTarget, CommandList, ConnectRequest, GameCommand, GameRequest,
    InterfacePanelAddress, LoadSceneCommand, PlayerIdentifier, RenderScreenOverlayCommand,
    SceneLoadMode, StandardAction,
};
use rules::{dispatch, mutations};
use screen_overlay::ScreenOverlay;
use serde_json::de;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info, info_span, instrument, warn};
use tutorial::tutorial_actions;
use user_action_data::{NewGameAction, NewGameDeck, UserAction};
use with_error::{fail, verify, WithError};

use crate::agent_response::HandleRequest;
use crate::{agent_response, debug};

/// Stores active channels for each user.
///
/// TODO: Clean this up on disconnect. This is quite easy to do with 'real' gRPC
/// but I haven't figured out how to do it with gRPC-web (which is just
/// fake-streaming over HTTP1). Unity doesn't support HTTP2 natively, but it's
/// possible to do it via a third party networking stack.
static CHANNELS: Lazy<DashMap<PlayerId, Sender<Result<CommandList, Status>>>> =
    Lazy::new(DashMap::new);

pub type ResponseInterceptor = fn(&CommandList);

/// Struct which implements our GRPC service
pub struct GameService {
    pub response_interceptor: Option<ResponseInterceptor>,
}

#[tonic::async_trait]
impl Spelldawn for GameService {
    type ConnectStream = ReceiverStream<Result<CommandList, Status>>;

    async fn connect(
        &self,
        request: Request<ConnectRequest>,
    ) -> Result<Response<Self::ConnectStream>, Status> {
        let mut db = SledDatabase { flush_on_write: false };
        let message = request.get_ref();
        let player_id = match player_id(&mut db, message.player_id.as_ref()) {
            Ok(player_id) => player_id,
            _ => return Err(Status::unauthenticated("PlayerId is required")),
        };

        let (tx, rx) = mpsc::channel(4);

        let result = handle_connect(&mut db, player_id);
        match result {
            Ok(commands) => {
                let names = commands.commands.iter().map(command_name).collect::<Vec<_>>();
                info!(?player_id, ?names, "Sending connection response");

                if let Err(error) = tx.send(Ok(commands)).await {
                    error!(?player_id, ?error, "Send Error!");
                    return Err(Status::internal(format!("Send Error: {error:#}")));
                }

                let result = agent_response::handle_request_if_active(
                    db,
                    request.get_ref().player_id.as_ref(),
                    HandleRequest::SendToPlayer,
                );
                if let Err(error) = result {
                    return Err(Status::internal(format!("Agent Error: {error:#}")));
                }
            }
            Err(error) => {
                error!(?player_id, ?error, "Connection Error!");
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
        let mut db = SledDatabase { flush_on_write: false };
        let response = handle_request(&mut db, request.get_ref());
        match response {
            Ok(response) => {
                if let Some(interceptor) = self.response_interceptor {
                    interceptor(&response.command_list);
                }

                send_player_response(response.opponent_response).await;

                if response.run_agent {
                    let result = agent_response::handle_request_if_active(
                        db,
                        request.get_ref().player_id.as_ref(),
                        HandleRequest::SendToPlayer,
                    );
                    if let Err(error) = result {
                        return Err(Status::internal(format!("Agent Error: {error:#}")));
                    }
                }

                Ok(Response::new(response.command_list))
            }
            Err(error) => {
                error!(?error, "Server Error!");
                Err(Status::internal(format!("Server Error: {error:#}")))
            }
        }
    }
}

/// Helper to perform the connect action from the unity plugin
pub fn connect(message: ConnectRequest) -> Result<CommandList> {
    let mut db = SledDatabase { flush_on_write: true };
    let player_id = player_id(&mut db, message.player_id.as_ref())?;
    handle_connect(&mut db, player_id)
}

/// Helper to perform an action from the unity plugin
pub fn perform_action(request: GameRequest) -> Result<CommandList> {
    let mut db = SledDatabase { flush_on_write: true };
    let response = handle_request(&mut db, &request)?;
    agent_response::handle_request_if_active(
        db,
        request.player_id.as_ref(),
        HandleRequest::PushQueue,
    )?;
    Ok(response.command_list)
}

/// A response to a given [GameRequest].
///
/// Returned from [handle_request] to support providing updates to different
/// players in a game.
#[derive(Debug, Clone, Default)]
pub struct GameResponse {
    /// Response to send to the user who made the initial game request.
    pub command_list: CommandList,
    /// Response to send to update opponent state.
    pub opponent_response: Option<(PlayerId, CommandList)>,
    /// True if this response should invoke AI Agent responses
    pub run_agent: bool,
}

impl GameResponse {
    pub fn from_commands(context: Option<ResponseContext>, command_list: Vec<Command>) -> Self {
        Self {
            command_list: core_ui::actions::command_list(context, command_list),
            opponent_response: None,
            run_agent: false,
        }
    }
}

/// Processes an incoming client request and returns a [GameResponse] describing
/// required updates to send to connected users.
pub fn handle_request(database: &mut impl Database, request: &GameRequest) -> Result<GameResponse> {
    let player_id = player_id(database, request.player_id.as_ref())?;
    let game_id = player_data::current_game_id(database.player(player_id)?);
    let client_action = request
        .action
        .as_ref()
        .with_error(|| "Action is required")?
        .action
        .as_ref()
        .with_error(|| "ClientAction is required")?;

    let response = match client_action {
        Action::StandardAction(standard_action) => handle_standard_action(
            database,
            player_id,
            game_id,
            &request.open_panels,
            standard_action,
        ),
        Action::FetchPanel(fetch_panel) => Ok(GameResponse::from_commands(
            game_id.map(ResponseContext::Game),
            vec![Command::UpdatePanels(routing::render_panel(
                &find_player(database, player_id)?,
                fetch_panel.panel_address.clone().with_error(|| "missing address")?,
            )?)],
        )),
        Action::DrawCard(_) => {
            let _span = info_span!("handle_draw_card").entered();
            warn!(?player_id, ?game_id, "Handling draw card action");
            handle_game_action(database, player_id, game_id, GameAction::DrawCard)
        }
        Action::PlayCard(action) => {
            let _span = info_span!("handle_play_card").entered();
            warn!(?action, ?player_id, ?game_id, "Handling play card action");
            let action =
                match adapters::server_card_id(action.card_id.with_error(|| "CardID expected")?)? {
                    ServerCardId::CardId(card_id) => {
                        GameAction::PlayCard(card_id, card_target(&action.target))
                    }
                    ServerCardId::AbilityId(ability_id) => {
                        GameAction::ActivateAbility(ability_id, card_target(&action.target))
                    }
                };
            handle_game_action(database, player_id, game_id, action)
        }
        Action::GainMana(_) => {
            let _span = info_span!("handle_gain_mana").entered();
            warn!(?player_id, ?game_id, "Handling gain mana action");
            handle_game_action(database, player_id, game_id, GameAction::GainMana)
        }
        Action::InitiateRaid(action) => {
            let _span = info_span!("handle_initiate_raid").entered();
            warn!(?action, ?player_id, ?game_id, "Handling initiate raid action");
            let room_id = adapters::room_id(action.room_id)?;
            handle_game_action(database, player_id, game_id, GameAction::InitiateRaid(room_id))
        }
        Action::LevelUpRoom(level_up) => {
            let _span = info_span!("handle_level_up_room").entered();
            warn!(?level_up, ?player_id, ?game_id, "Handling level up room action");
            let room_id = adapters::room_id(level_up.room_id)?;
            handle_game_action(database, player_id, game_id, GameAction::LevelUpRoom(room_id))
        }
        Action::SpendActionPoint(_) => {
            handle_game_action(database, player_id, game_id, GameAction::SpendActionPoint)
        }
    }?;

    let commands = response.command_list.commands.iter().map(command_name).collect::<Vec<_>>();

    info!(?player_id, ?commands, "Sending action response");

    Ok(response)
}

/// Sets up the game state for a game connection request.
#[instrument(skip(database))]
pub fn handle_connect(database: &mut impl Database, player_id: PlayerId) -> Result<CommandList> {
    let (player, is_new_player) = match database.player(player_id)? {
        Some(p) => (p, false),
        None => (create_new_player(database, player_id)?, true),
    };
    warn!(?player_id, "handle_connect");

    let mut commands = vec![];
    let mut context: Option<ResponseContext> = None;
    match (&player.status, &player.adventure) {
        (Some(PlayerStatus::Playing(game_id)), _) => {
            if database.has_game(*game_id)? {
                context = Some(ResponseContext::Game(*game_id));
                let game = database.game(*game_id)?;
                let side = user_side(player_id, &game)?;
                commands.extend(render::connect(&game, side)?);
                routing::render_panels(&mut commands, &player, routing::game_panels())?;
            } else {
                fail!("Game not found: {:?}", game_id)
            }
        }
        (Some(PlayerStatus::RequestedGame(_)), _) => todo!("Not implemented"),
        (None, Some(adventure_state)) => {
            context = Some(ResponseContext::Adventure(adventure_state.id));
            commands.extend(adventure_display::render(adventure_state)?);
            routing::render_panels(
                &mut commands,
                &player,
                routing::adventure_panels(adventure_state),
            )?;
        }
        (None, None) => {
            info!("Loading Main scene");
            commands.push(Command::LoadScene(LoadSceneCommand {
                scene_name: "Main".to_string(),
                mode: SceneLoadMode::Single.into(),
                skip_if_current: true,
            }));
            routing::render_panels(&mut commands, &player, routing::main_menu_panels())?;
            commands.push(Panels::open(PanelAddress::MainMenu).into());
            if is_new_player {
                commands.push(Panels::open(PanelAddress::Disclaimer).into());
            }
        }
    }

    commands.push(update_navbar(&player));
    Ok(core_ui::actions::command_list(context, commands))
}

fn handle_new_adventure(
    database: &mut impl Database,
    player_id: PlayerId,
    side: Side,
) -> Result<GameResponse> {
    let mut player = database.player(player_id)?.with_error(|| "Player not found")?;
    player.adventure =
        Some(adventure_generator::new_adventure(AdventureConfiguration::new(player_id, side)));
    database.write_player(&player)?;
    Ok(GameResponse::from_commands(
        None,
        vec![Command::LoadScene(LoadSceneCommand {
            scene_name: "World".to_string(),
            mode: SceneLoadMode::Single.into(),
            skip_if_current: true,
        })],
    ))
}

/// Creates a new default [GameState], deals opening hands, and writes its value
/// to the database.
fn handle_new_game(
    database: &mut impl Database,
    player_id: PlayerId,
    action: NewGameAction,
) -> Result<GameResponse> {
    let debug_options = action.debug_options.unwrap_or_default();
    let opponent_id = action.opponent;
    let mut player = database.player(player_id)?.with_error(|| "Player not found")?;
    let user_deck = find_deck(&player, action.deck)?;
    let opponent_deck =
        if let Some(deck) = requested_deck(database, opponent_id, user_deck.side.opponent())? {
            deck
        } else {
            player.status = Some(PlayerStatus::RequestedGame(action));
            database.write_player(&player)?;
            // TODO: Implement this
            return Ok(GameResponse::from_commands(None, vec![]));
        };

    let (user_side, opponent_side) = (user_deck.side, opponent_deck.side);
    let (overlord_deck, champion_deck, overlord_id, champion_id) = match (user_side, opponent_side)
    {
        (Side::Overlord, Side::Champion) => (user_deck, opponent_deck, player_id, opponent_id),
        (Side::Champion, Side::Overlord) => (opponent_deck, user_deck, opponent_id, player_id),
        _ => fail!("Deck side mismatch!"),
    };

    let game_id = if let Some(id) = debug_options.override_game_id {
        id
    } else {
        database.generate_game_id()?
    };
    info!(?game_id, "Creating new game");

    let mut game = GameState::new(
        game_id,
        overlord_id,
        overlord_deck,
        champion_id,
        champion_deck,
        GameConfiguration {
            deterministic: debug_options.deterministic,
            scripted_tutorial: action.tutorial,
            ..GameConfiguration::default()
        },
    );

    dispatch::populate_delegate_cache(&mut game);
    mutations::deal_opening_hands(&mut game)?;

    if game.data.config.scripted_tutorial {
        // Start tutorial if needed
        tutorial_actions::handle_sequence_game_action(&mut game, None)?;
    }

    database.write_game(&game)?;

    player.status = Some(PlayerStatus::Playing(game_id));
    database.write_player(&player)?;

    if let PlayerId::Database(_) = opponent_id {
        let mut opponent = database.player(opponent_id)?.with_error(|| "Opponent not found")?;
        opponent.status = Some(PlayerStatus::Playing(game_id));
        database.write_player(&opponent)?;
    }

    let context = Some(ResponseContext::Game(game_id));
    Ok(GameResponse {
        command_list: core_ui::actions::command_list(context, render::connect(&game, user_side)?),
        opponent_response: Some((
            opponent_id,
            core_ui::actions::command_list(context, render::connect(&game, opponent_side)?),
        )),
        run_agent: false,
    })
}

fn find_deck(player: &PlayerData, deck: NewGameDeck) -> Result<Deck> {
    Ok(match deck {
        NewGameDeck::DeckId(id) => player.deck(id)?.clone(),
        NewGameDeck::NamedDeck(name) => decklists::named_deck(name),
    })
}

fn handle_leave_game(database: &mut impl Database, player_id: PlayerId) -> Result<GameResponse> {
    let mut player = database.player(player_id)?.with_error(|| "Player not found")?;
    player.status = None;
    database.write_player(&player)?;
    Ok(GameResponse::from_commands(
        Some(ResponseContext::LeaveGame),
        vec![Command::LoadScene(LoadSceneCommand {
            scene_name: "Main".to_string(),
            mode: SceneLoadMode::Single.into(),
            skip_if_current: true,
        })],
    ))
}

fn handle_leave_adventure(state: &mut PlayerData) -> Result<Vec<Command>> {
    state.adventure = None;
    Ok(vec![Command::LoadScene(LoadSceneCommand {
        scene_name: "Main".to_string(),
        mode: SceneLoadMode::Single.into(),
        skip_if_current: true,
    })])
}

/// Looks up the deck the `player_id` player has requested to use for a new game
fn requested_deck(
    database: &impl Database,
    player_id: PlayerId,
    side: Side,
) -> Result<Option<Deck>> {
    Ok(match player_id {
        PlayerId::Database(_) => {
            let player = find_player(database, player_id)?;
            match player.status {
                Some(PlayerStatus::RequestedGame(action)) => Some(find_deck(&player, action.deck)?),
                _ => None,
            }
        }
        // TODO: Each named player should have their own decklist
        PlayerId::Named(name) => match name {
            NamedPlayer::TutorialOpponent => Some(decklists::TUTORIAL_OVERLORD.clone()),
            NamedPlayer::DebugChampion => Some(decklists::CANONICAL_CHAMPION.clone()),
            NamedPlayer::DebugOverlord => Some(decklists::CANONICAL_OVERLORD.clone()),
            _ => Some(decklists::basic_deck(side)),
        },
    })
}

/// Queries the [GameState] for a game from the [Database] and then invokes the
/// [actions::handle_game_action] function to apply the provided [UserAction].
///
/// Converts the resulting [GameState] into a series of client updates for both
/// players in the form of a [GameResponse] and then writes the new game state
/// back to the database
pub fn handle_game_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    action: GameAction,
) -> Result<GameResponse> {
    handle_custom_action(database, player_id, game_id, |game, user_side| {
        tutorial_actions::handle_game_action(game, Some(&action))?;
        actions::handle_game_action(game, user_side, action)?;
        tutorial_actions::handle_game_action(game, None)
    })
}

/// Custom version of `handle_action` which accepts a function allowing
/// arbitrary mutation of the [GameState].
pub fn handle_custom_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    function: impl Fn(&mut GameState, Side) -> Result<()>,
) -> Result<GameResponse> {
    // TODO: Use transactions?
    let mut game = find_game(database, game_id)?;
    let user_side = user_side(player_id, &game)?;
    function(&mut game, user_side)?;

    let user_result = render::render_updates(&game, user_side)?;
    let opponent_id = game.player(user_side.opponent()).id;
    let context = game_id.map(ResponseContext::Game);

    let channel_response = Some((
        opponent_id,
        core_ui::actions::command_list(
            context,
            render::render_updates(&game, user_side.opponent())?,
        ),
    ));
    database.write_game(&game)?;

    Ok(GameResponse {
        command_list: core_ui::actions::command_list(context, user_result),
        opponent_response: channel_response,
        run_agent: true,
    })
}

/// Allows mutation of a player's data outside of an active game ([PlayerData]).
pub fn handle_player_action(
    database: &mut impl Database,
    player_id: PlayerId,
    context: Option<ResponseContext>,
    function: impl Fn(&mut PlayerData) -> Result<Vec<Command>>,
) -> Result<GameResponse> {
    let mut player = find_player(database, player_id)?;
    let response = function(&mut player)?;
    database.write_player(&player)?;
    Ok(GameResponse::from_commands(context, response))
}

/// Allows mutation of a player's data outside of an active game ([PlayerData]).
pub fn with_adventure(
    database: &mut impl Database,
    player_id: PlayerId,
    function: impl Fn(&mut AdventureState) -> Result<()>,
) -> Result<GameResponse> {
    let mut player = find_player(database, player_id)?;
    let adventure_state = player.adventure.as_mut().with_error(|| "Expected active adventure")?;
    let id = adventure_state.id;
    function(adventure_state)?;
    let commands = adventure_display::render(adventure_state)?;
    database.write_player(&player)?;
    Ok(GameResponse::from_commands(Some(ResponseContext::Adventure(id)), commands))
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

/// Parses the serialized payload in a [StandardAction] and dispatches to the
/// correct handler. Updates interface panels provided in 'open_panels'.
fn handle_standard_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    open_panels: &Vec<InterfacePanelAddress>,
    standard_action: &StandardAction,
) -> Result<GameResponse> {
    verify!(!standard_action.payload.is_empty(), "Empty action payload received");
    let action: UserAction = de::from_slice(&standard_action.payload)
        .with_error(|| "Failed to deserialize action payload")?;
    let _span = info_span!("handle_standard_action", ?player_id, ?game_id, ?action).entered();
    warn!(?action, ?player_id, ?game_id, "Action");

    let mut result = match action {
        UserAction::NewAdventure(side) => handle_new_adventure(database, player_id, side),
        UserAction::AdventureAction(action) => with_adventure(database, player_id, |state| {
            adventure_actions::handle_adventure_action(state, &action)
        }),
        UserAction::LeaveAdventure => handle_player_action(
            database,
            player_id,
            Some(ResponseContext::LeaveAdventure),
            handle_leave_adventure,
        ),
        UserAction::NewGame(new_game_action) => {
            handle_new_game(database, player_id, new_game_action)
        }
        UserAction::LeaveGame => handle_leave_game(database, player_id),
        UserAction::Debug(debug_action) => {
            debug::handle_debug_action(database, player_id, game_id, debug_action)
        }
        UserAction::GameAction(a) => handle_game_action(database, player_id, game_id, a),
        UserAction::DeckEditorAction(a) => {
            handle_player_action(database, player_id, None, |player| {
                deck_editor_actions::handle(player, a)?;
                Ok(vec![])
            })
        }
    }?;

    let player = find_player(database, player_id)?;
    for address in open_panels {
        result.command_list.commands.push(GameCommand {
            command: Some(Command::UpdatePanels(routing::render_panel(&player, address.clone())?)),
        });
    }
    result.command_list.commands.push(GameCommand { command: Some(update_navbar(&player)) });

    Ok(result)
}

fn update_navbar(player: &PlayerData) -> Command {
    Command::RenderScreenOverlay(RenderScreenOverlayCommand {
        node: ScreenOverlay::new(player).build(),
    })
}

/// Look up the state for a game which is expected to exist and assigns an
/// [UpdateTracker] to it for the duration of this request.
pub fn find_game(database: &impl Database, game_id: Option<GameId>) -> Result<GameState> {
    let id = game_id.as_ref().with_error(|| "GameId not provided!")?;
    let mut game = database.game(*id)?;
    game.updates = UpdateTracker::new(if game.data.config.simulation {
        Updates::Ignore
    } else {
        Updates::Push
    });

    Ok(game)
}

/// Writes the default initial state for a new player to the provided database
fn create_new_player(database: &mut impl Database, player_id: PlayerId) -> Result<PlayerData> {
    let result = PlayerData {
        id: player_id,
        status: None,
        adventure: None,
        tutorial: TutorialData::default(),
    };
    database.write_player(&result)?;
    Ok(result)
}

/// Look up the [PlayerData] for a player, or creates a new instance if none
/// already exists.
pub fn find_player(database: &impl Database, player_id: PlayerId) -> Result<PlayerData> {
    Ok(database.player(player_id)?.unwrap_or_else(|| PlayerData::new(player_id)))
}

/// Turns an `&Option<PlayerIdentifier>` into a [PlayerId], or returns an error
/// if the input is `None`.
pub fn player_id(
    database: &mut impl Database,
    identifier: Option<&PlayerIdentifier>,
) -> Result<PlayerId> {
    database
        .adapt_player_identifier(identifier.as_ref().with_error(|| "Expected player identifier")?)
}

/// Returns the [Side] the indicated user is representing in this game
pub fn user_side(player_id: PlayerId, game: &GameState) -> Result<Side> {
    if player_id == game.champion.id {
        Ok(Side::Champion)
    } else if player_id == game.overlord.id {
        Ok(Side::Overlord)
    } else {
        fail!("User {:?} is not a participant in game {:?}", player_id, game.id)
    }
}

/// Get a display name for a command. Used for debugging.
pub fn command_name(command: &GameCommand) -> &'static str {
    command.command.as_ref().map_or("None", |c| match c {
        Command::Debug(_) => "Debug",
        Command::Delay(_) => "Delay",
        Command::UpdatePanels(_) => "UpdatePanels",
        Command::TogglePanel(_) => "TogglePanel",
        Command::UpdateGameView(_) => "UpdateGameView",
        Command::VisitRoom(_) => "VisitRoom",
        Command::MoveGameObjects(_) => "MoveGameObjects",
        Command::PlaySound(_) => "PlaySound",
        Command::SetMusic(_) => "SetMusic",
        Command::FireProjectile(_) => "FireProjectile",
        Command::PlayEffect(_) => "PlayEffect",
        Command::DisplayGameMessage(_) => "DisplayGameMessage",
        Command::SetGameObjectsEnabled(_) => "SetGameObjectsEnabled",
        Command::DisplayRewards(_) => "DisplayRewards",
        Command::LoadScene(_) => "LoadScene",
        Command::CreateTokenCard(_) => "CreateTokenCard",
        Command::UpdateWorldMap(_) => "UpdateWorldMap",
        Command::RenderScreenOverlay(_) => "RenderScreenOverlay",
        Command::UpdateInterface(_) => "UpdateInterface",
        Command::Conditional(_) => "Conditional",
        Command::InfoZoom(_) => "InfoZoom",
    })
}

fn card_target(target: &Option<CardTarget>) -> game_actions::CardTarget {
    target.as_ref().map_or(game_actions::CardTarget::None, |t| {
        t.card_target.as_ref().map_or(game_actions::CardTarget::None, |t2| match t2 {
            card_target::CardTarget::RoomId(room_id) => adapters::room_id(*room_id)
                .map_or(game_actions::CardTarget::None, game_actions::CardTarget::Room),
        })
    })
}
