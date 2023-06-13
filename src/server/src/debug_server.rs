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

use std::mem;

use anyhow::Result;
use core_ui::actions::InterfaceAction;
use database::Database;
use game_data::game::GameState;
use game_data::player_name::{NamedPlayer, PlayerId};
use game_data::primitives::{GameId, Side};
use player_data::PlayerStatus;
use protos::spelldawn::client_debug_command::DebugCommand;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{ClientAction, ClientDebugCommand, LoadSceneCommand, SceneLoadMode};
use rules::mana;
use user_action_data::{
    DebugAction, NamedDeck, NewGameAction, NewGameDebugOptions, NewGameDeck, UserAction,
};
use with_error::WithError;

use crate::server_data::{ClientData, GameResponse, RequestData};
use crate::{game_server, requests};

pub async fn handle_debug_action(
    database: &impl Database,
    data: &RequestData,
    action: &DebugAction,
) -> Result<GameResponse> {
    match action {
        DebugAction::NewGame(side) => create_debug_game(data, *side),
        DebugAction::JoinGame(side) => {
            let game_id = GameId::new_from_u128(0);
            let mut game = requests::fetch_game(database, Some(game_id)).await?;
            match side {
                Side::Overlord => game.overlord.id = data.player_id,
                Side::Champion => game.champion.id = data.player_id,
            }
            let result = reload_scene(data, &game);
            database.write_game(&game).await?;
            let mut player = requests::fetch_player(database, data.player_id).await?;
            player.status = Some(PlayerStatus::Playing(game_id));
            database.write_player(&player).await?;
            result
        }
        DebugAction::FlipViewpoint => {
            requests::with_game(database, data, |game| {
                mem::swap(&mut game.champion.id, &mut game.overlord.id);
                reload_scene(data, game)
            })
            .await
        }
        DebugAction::AddMana(amount) => {
            game_server::update_game(database, data, |game, user_side| {
                mana::gain(game, user_side, *amount);
                Ok(())
            })
            .await
        }
        DebugAction::AddActionPoints(amount) => {
            game_server::update_game(database, data, |game, user_side| {
                game.player_mut(user_side).actions += amount;
                Ok(())
            })
            .await
        }
        DebugAction::AddScore(amount) => {
            game_server::update_game(database, data, |game, user_side| {
                game.player_mut(user_side).score += amount;
                Ok(())
            })
            .await
        }
        DebugAction::SaveState(index) => {
            let mut game = requests::fetch_game(database, data.game_id).await?;
            let game_id = GameId::new_from_u128(100 + index);
            game.id = game_id;
            database.write_game(&game).await?;
            Ok(GameResponse::new(ClientData::propagate(data)))
        }
        DebugAction::LoadState(index) => {
            let game_id = data.game_id.with_error(|| "Expected game_id")?;
            let saved_id = GameId::new_from_u128(100 + index);
            let mut game = requests::fetch_game(database, Some(saved_id)).await?;
            game.id = game_id;
            let result = reload_scene(data, &game);
            database.write_game(&game).await?;
            result
        }
        DebugAction::SetNamedPlayer(side, name) => {
            game_server::update_game(database, data, |game, _| {
                game.player_mut(*side).id = PlayerId::Named(*name);
                Ok(())
            })
            .await
        }
    }
}

fn create_debug_game(data: &RequestData, side: Side) -> Result<GameResponse> {
    Ok(GameResponse::new(ClientData::propagate(data)).commands(vec![Command::Debug(
        ClientDebugCommand {
            debug_command: Some(DebugCommand::InvokeAction(ClientAction {
                action: Some(
                    UserAction::NewGame(NewGameAction {
                        opponent: PlayerId::Named(match side {
                            Side::Overlord => NamedPlayer::DebugChampion,
                            Side::Champion => NamedPlayer::DebugOverlord,
                        }),
                        deck: match side {
                            Side::Overlord => NewGameDeck::NamedDeck(NamedDeck::CanonicalOverlord),
                            Side::Champion => NewGameDeck::NamedDeck(NamedDeck::CanonicalChampion),
                        },
                        debug_options: Some(NewGameDebugOptions {
                            deterministic: false,
                            override_game_id: Some(GameId::new_from_u128(0)),
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
