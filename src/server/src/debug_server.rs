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
use std::mem;
use std::sync::{Mutex, OnceLock};

use ::panels::add_to_zone_panel::AddToZonePanel;
use anyhow::Result;
use core_ui::actions::InterfaceAction;
use core_ui::panels;
use database::Database;
use display::render;
use game_data::card_name::{CardName, CardVariant};
use game_data::card_state::CardPosition;
use game_data::game_actions::{GameAction, GameStateAction};
use game_data::game_state::{GameState, MulliganDecision};
use game_data::player_name::{AIPlayer, PlayerId};
use game_data::primitives::{CardId, GameId, RoomId, RoomLocation, Side};
use panel_address::Panel;
use player_data::PlayerStatus;
use protos::spelldawn::client_debug_command::DebugCommand;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{ClientAction, ClientDebugCommand, LoadSceneCommand, SceneLoadMode};
use rules::mutations::SummonMinion;
use rules::{dispatch, mana, mutations};
use tracing::debug;
use ulid::Ulid;
use user_action_data::{
    DebugAction, DebugScenario, NamedDeck, NewGameAction, NewGameDebugOptions, NewGameDeck,
    UserAction,
};
use with_error::{fail, WithError};

use crate::server_data::{ClientData, GameResponse, RequestData};
use crate::{adventure_server, requests};

fn current_game_id() -> &'static Mutex<Option<GameId>> {
    static GAME_ID: OnceLock<Mutex<Option<GameId>>> = OnceLock::new();
    GAME_ID.get_or_init(|| Mutex::new(None))
}

pub async fn handle_debug_action(
    database: &impl Database,
    data: &RequestData,
    action: &DebugAction,
    request_fields: &HashMap<String, String>,
) -> Result<GameResponse> {
    match action {
        DebugAction::NewGame(side) => create_debug_game(data, *side),
        DebugAction::JoinGame(side) => {
            let game_id = match current_game_id().lock() {
                Ok(id) => id,
                Err(_) => fail!("Unable to acquire lock, another holder panicked"),
            }
            .with_error(|| "Current debug game id not found")?;
            let mut game = requests::fetch_game(database, Some(game_id)).await?;
            match side {
                Side::Overlord => game.overlord.id = data.player_id,
                Side::Champion => game.champion.id = data.player_id,
            }
            let result = reload_scene(data, &game);
            database.write_game(&game).await?;
            let mut player = requests::fetch_player(database, data.player_id).await?;
            player.status = Some(PlayerStatus::Playing(game_id, *side));
            database.write_player(&player).await?;
            result
        }
        DebugAction::FlipViewpoint => {
            let mut game = requests::fetch_game(
                database,
                Some(data.game_id.with_error(|| "Expected game id")?),
            )
            .await?;
            let mut player = requests::fetch_player(database, data.player_id).await?;
            if data.player_id == game.overlord.id {
                player.status = Some(PlayerStatus::Playing(game.id, Side::Champion));
                database.write_player(&player).await?;
            } else {
                player.status = Some(PlayerStatus::Playing(game.id, Side::Overlord));
                database.write_player(&player).await?;
            }
            mem::swap(&mut game.champion.id, &mut game.overlord.id);
            database.write_game(&game).await?;

            reload_scene(data, &game)
        }
        DebugAction::AddMana(amount) => {
            debug_update_game(database, data, |game, user_side| {
                mana::gain(game, user_side, *amount);
                Ok(())
            })
            .await
        }
        DebugAction::AddActionPoints(amount) => {
            debug_update_game(database, data, |game, user_side| {
                game.player_mut(user_side).actions += amount;
                Ok(())
            })
            .await
        }
        DebugAction::AddScore(amount) => {
            debug_update_game(database, data, |game, user_side| {
                mutations::score_points(game, user_side, *amount)?;
                Ok(())
            })
            .await
        }
        DebugAction::SaveGameState(index) => {
            let mut game = requests::fetch_game(database, data.game_id).await?;
            let game_id = GameId::new_from_u128(100 + index);
            game.id = game_id;
            database.write_game(&game).await?;
            Ok(GameResponse::new(ClientData::propagate(data)))
        }
        DebugAction::LoadGameState(index) => {
            let game_id = data.game_id.with_error(|| "Expected game_id")?;
            let saved_id = GameId::new_from_u128(100 + index);
            let mut game = requests::fetch_game(database, Some(saved_id)).await?;
            game.id = game_id;
            let result = reload_scene(data, &game);
            database.write_game(&game).await?;
            result
        }
        DebugAction::SavePlayerState(index) => {
            let mut player = requests::fetch_player(database, data.player_id).await?;
            let player_id = PlayerId::Database(Ulid(*index));
            player.id = player_id;
            database.write_player(&player).await?;
            Ok(GameResponse::new(ClientData::propagate(data)))
        }
        DebugAction::LoadPlayerState(index) => {
            let saved_id = PlayerId::Database(Ulid(*index));
            let mut player = requests::fetch_player(database, saved_id).await?;
            player.id = data.player_id;
            let result = reload_world_scene(data);
            database.write_player(&player).await?;
            Ok(result)
        }
        DebugAction::SetNamedPlayer(side, name) => {
            debug_update_game(database, data, |game, _| {
                game.player_mut(*side).id = PlayerId::AI(*name);
                Ok(())
            })
            .await
        }
        DebugAction::AddCoins(coins) => {
            adventure_server::update_adventure(database, data, |state| {
                state.coins += *coins;
                Ok(())
            })
            .await
        }
        DebugAction::AddCurses(amount) => {
            debug_update_game(database, data, |game, _| {
                mutations::give_curses(game, *amount)?;
                Ok(())
            })
            .await
        }
        DebugAction::RemoveCurses(amount) => {
            debug_update_game(database, data, |game, _| {
                mutations::remove_curses(game, *amount)?;
                Ok(())
            })
            .await
        }
        DebugAction::FilterCardList(position, metadata) => {
            let input = request_fields.get("CardVariant").with_error(|| "Expected CardVariant")?;
            Ok(GameResponse::new(ClientData::propagate(data)).command(panels::update(
                AddToZonePanel::new(input, *position, *metadata).build_panel().unwrap(),
            )))
        }
        DebugAction::AddToZone(card_name, position) => {
            debug_update_game(database, data, |game, user_side| {
                if let Some(top_of_deck) =
                    mutations::realize_top_of_deck(game, user_side, 1)?.get(0)
                {
                    mutations::overwrite_card(game, *top_of_deck, *card_name)?;
                    if matches!(position, CardPosition::Hand(s) if *s == user_side) {
                        mutations::draw_cards(game, user_side, 1)?;
                    } else if matches!(position, CardPosition::DiscardPile(_)) {
                        mutations::discard_card(game, *top_of_deck)?;
                    } else {
                        mutations::move_card(game, *top_of_deck, *position)?;
                    }
                }
                Ok(())
            })
            .await
        }
        DebugAction::ApplyScenario(scenario) => {
            debug_update_game(database, data, |game, _| apply_scenario(*scenario, data, game))
                .await?;

            let game = requests::fetch_game(database, data.game_id).await?;
            let mut player = requests::fetch_player(database, data.player_id).await?;
            player.status = Some(PlayerStatus::Playing(game.id, scenario.side()));
            database.write_player(&player).await?;
            reload_scene(data, &game)
        }
        DebugAction::DebugUndo => {
            debug_update_game(database, data, |game, _| {
                let new_state = game
                    .undo_tracker
                    .as_mut()
                    .with_error(|| "Expected undo_tracker")?
                    .undo
                    .take()
                    .with_error(|| "Expected undo state")?;
                *game = *new_state;
                Ok(())
            })
            .await
        }
    }
}

fn create_debug_game(data: &RequestData, side: Side) -> Result<GameResponse> {
    let id = GameId::new(Ulid::new());
    let mut current_game_id = match current_game_id().lock() {
        Ok(id) => id,
        Err(_) => fail!("Unable to acquire lock, another holder panicked"),
    };
    debug!(?id, "Creating debug game");
    let _ = current_game_id.insert(id);
    Ok(GameResponse::new(ClientData::propagate(data)).commands(vec![Command::Debug(
        ClientDebugCommand {
            debug_command: Some(DebugCommand::InvokeAction(ClientAction {
                action: Some(
                    UserAction::NewGame(NewGameAction {
                        opponent: PlayerId::AI(match side {
                            Side::Overlord => AIPlayer::DebugChampion,
                            Side::Champion => AIPlayer::DebugOverlord,
                        }),
                        deck: match side {
                            Side::Overlord => NewGameDeck::NamedDeck(NamedDeck::CanonicalOverlord),
                            Side::Champion => NewGameDeck::NamedDeck(NamedDeck::CanonicalChampion),
                        },
                        debug_options: Some(NewGameDebugOptions {
                            deterministic: false,
                            override_game_id: Some(id),
                        }),
                        tutorial: false,
                    })
                    .as_client_action(),
                ),
            })),
        },
    )]))
}

fn reload_scene(data: &RequestData, game: &GameState) -> Result<GameResponse> {
    let command = Command::LoadScene(LoadSceneCommand {
        scene_name: "Game".to_string(),
        mode: SceneLoadMode::Single as i32,
        skip_if_current: false,
    });
    let user_side = game.player_side(data.player_id)?;
    let opponent_id = game.player(user_side.opponent()).id;
    Ok(GameResponse::new(ClientData::with_game_id(data, Some(game.id)))
        .command(command.clone())
        .opponent_response(opponent_id, vec![command]))
}

fn reload_world_scene(data: &RequestData) -> GameResponse {
    let command = Command::LoadScene(LoadSceneCommand {
        scene_name: "World".to_string(),
        mode: SceneLoadMode::Single as i32,
        skip_if_current: false,
    });
    GameResponse::new(ClientData::propagate(data)).command(command)
}

fn apply_scenario(scenario: DebugScenario, data: &RequestData, game: &mut GameState) -> Result<()> {
    *game = scenario_game(game, data, scenario.side())?;

    match scenario {
        DebugScenario::NewGameOverlord => {}
        DebugScenario::NewGameChampion => {}
        DebugScenario::VsInfernalMinionAndScheme => {
            create_at_position(
                game,
                CardName::TestScheme3_10,
                CardPosition::Room(RoomId::RoomE, RoomLocation::Occupant),
            )?;
            let minion_id = create_at_position(
                game,
                CardName::TestInfernalMinion,
                CardPosition::Room(RoomId::RoomE, RoomLocation::Defender),
            )?;
            mutations::summon_minion(game, minion_id, SummonMinion::IgnoreCosts)?;
        }
    }
    Ok(())
}

fn scenario_game(game: &GameState, data: &RequestData, side: Side) -> Result<GameState> {
    let mut result = GameState::new(
        game.id,
        if side == Side::Overlord { data.player_id } else { PlayerId::AI(AIPlayer::NoAction) },
        decklists::CANONICAL_OVERLORD.clone(),
        if side == Side::Champion { data.player_id } else { PlayerId::AI(AIPlayer::NoAction) },
        decklists::CANONICAL_CHAMPION.clone(),
        game.info.config,
    );
    dispatch::populate_delegate_cache(&mut result);
    actions::handle_game_action(
        &mut result,
        Side::Overlord,
        &GameAction::GameStateAction(GameStateAction::MulliganDecision(MulliganDecision::Keep)),
    )?;
    actions::handle_game_action(
        &mut result,
        Side::Champion,
        &GameAction::GameStateAction(GameStateAction::MulliganDecision(MulliganDecision::Keep)),
    )?;

    if side == Side::Champion {
        for _ in 0..3 {
            actions::handle_game_action(
                &mut result,
                Side::Overlord,
                &GameAction::SpendActionPoint,
            )?;
        }

        actions::handle_game_action(
            &mut result,
            Side::Overlord,
            &GameAction::GameStateAction(GameStateAction::EndTurnAction),
        )?;
    }

    Ok(result)
}

fn create_at_position(
    game: &mut GameState,
    card: CardName,
    position: CardPosition,
) -> Result<CardId> {
    let side = rules::get(CardVariant::standard(card)).side;
    let card_id =
        *mutations::realize_top_of_deck(game, side, 1)?.get(0).with_error(|| "Deck is empty")?;
    mutations::overwrite_card(game, card_id, CardVariant::standard(card))?;
    mutations::move_card(game, card_id, position)?;
    Ok(card_id)
}

/// Applies a game mutation and produces a snapshot of the resulting game state
/// to send to both players.
async fn debug_update_game(
    database: &impl Database,
    data: &RequestData,
    function: impl Fn(&mut GameState, Side) -> Result<()>,
) -> Result<GameResponse> {
    requests::with_game(database, data, |game| {
        let user_side = game.player_side(data.player_id)?;
        function(game, user_side)?;

        let user_result = render::render_updates(game, user_side)?;
        let opponent_id = game.player(user_side.opponent()).id;
        let opponent_commands = render::render_updates(game, user_side.opponent())?;

        Ok(GameResponse::new(ClientData::with_game_id(data, Some(game.id)))
            .commands(user_result)
            .opponent_response(opponent_id, opponent_commands))
    })
    .await
}
