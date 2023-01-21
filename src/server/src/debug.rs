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
use core_ui::actions::InterfaceAction;
use core_ui::panels::Panels;
use data::card_name::CardName;
use data::game::GameState;
use data::player_name::{NamedPlayer, PlayerId};
use data::primitives::{GameId, Side};
use data::user_actions::{
    DebugAction, NamedDeck, NewGameAction, NewGameDebugOptions, NewGameDeck, UserAction,
};
use database::Database;
use panel_address::PanelAddress;
use protos::spelldawn::client_debug_command::DebugCommand;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{ClientAction, ClientDebugCommand, LoadSceneCommand, SceneLoadMode};
use rules::mana;
use with_error::WithError;

use crate::requests;
use crate::requests::GameResponse;

pub fn handle_debug_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    action: DebugAction,
) -> Result<GameResponse> {
    let close = Panels::close(PanelAddress::DebugPanel);
    match action {
        DebugAction::NewGame(side) => Ok(GameResponse::from_commands(vec![
            Command::Debug(ClientDebugCommand {
                debug_command: Some(DebugCommand::InvokeAction(ClientAction {
                    action: Some(
                        UserAction::NewGame(NewGameAction {
                            opponent: PlayerId::Named(NamedPlayer::NoAction),
                            deck: match side {
                                Side::Overlord => {
                                    NewGameDeck::NamedDeck(NamedDeck::CanonicalOverlord)
                                }
                                Side::Champion => {
                                    NewGameDeck::NamedDeck(NamedDeck::CanonicalChampion)
                                }
                            },
                            debug_options: Some(NewGameDebugOptions {
                                deterministic: false,
                                override_game_id: Some(GameId::new(0)),
                            }),
                            tutorial: false,
                        })
                        .as_client_action(),
                    ),
                })),
            }),
            close.into(),
        ])),
        DebugAction::JoinGame => {
            let mut game = requests::find_game(database, Some(GameId::new(0)))?;
            if matches!(game.overlord.id, PlayerId::Named(_)) {
                game.overlord.id = player_id;
            } else {
                game.champion.id = player_id;
            }
            database.write_game(&game)?;
            load_scene()
        }
        DebugAction::FlipViewpoint => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                let opponent_id = game.player(user_side.opponent()).id;
                game.player_mut(user_side.opponent()).id = player_id;
                game.player_mut(user_side).id = opponent_id;
                Ok(())
            })?;
            load_scene()
        }
        DebugAction::AddMana(amount) => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                mana::gain(game, user_side, amount);
                Ok(())
            })
        }
        DebugAction::AddActionPoints(amount) => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).actions += amount;
                Ok(())
            })
        }
        DebugAction::AddScore(amount) => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).score += amount;
                Ok(())
            })
        }
        DebugAction::SaveState(index) => {
            let mut game = load_game(database, game_id)?;
            game.id = GameId::new(u64::MAX - index);
            database.write_game(&game)?;
            Ok(GameResponse::from_commands(vec![]))
        }
        DebugAction::LoadState(index) => {
            let mut game = database.game(GameId::new(u64::MAX - index))?;
            game.id = game_id.with_error(|| "Expected GameId")?;
            database.write_game(&game)?;
            Ok(GameResponse::from_commands(vec![Command::LoadScene(LoadSceneCommand {
                scene_name: "Game".to_string(),
                mode: SceneLoadMode::Single.into(),
                skip_if_current: false,
            })]))
        }
        DebugAction::SetNamedPlayer(side, name) => {
            requests::handle_custom_action(database, player_id, game_id, |game, _| {
                game.player_mut(side).id = PlayerId::Named(name);
                Ok(())
            })
        }
        DebugAction::FullCollection => {
            requests::handle_player_action(database, player_id, |player| {
                for name in enum_iterator::all::<CardName>() {
                    if !name.is_test_card() && !name.is_null_identity() {
                        player.adventure_mut()?.collection.insert(name, 3);
                    }
                }
                Ok(vec![])
            })
        }
    }
}

fn load_scene() -> Result<GameResponse> {
    Ok(GameResponse::from_commands(vec![Command::LoadScene(LoadSceneCommand {
        scene_name: "Game".to_string(),
        mode: SceneLoadMode::Single as i32,
        skip_if_current: false,
    })]))
}

fn load_game(database: &mut impl Database, game_id: Option<GameId>) -> Result<GameState> {
    database.game(game_id.with_error(|| "GameId is required")?)
}
