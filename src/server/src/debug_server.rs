// Copyright © Riftcaller 2021-present

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
use std::sync::atomic::Ordering;

use ::panels::add_to_zone_panel::AddToZonePanel;
use anyhow::Result;
use core_data::game_primitives::{
    AbilityId, AbilityIndex, CardId, CardPlayId, GameId, InitiatedBy, RoomId, RoomLocation, Side,
};
use core_ui::actions::InterfaceAction;
use core_ui::panels;
use database::Database;
use display::render;
use game_data::card_name::{CardName, CardVariant};
use game_data::card_state::CardPosition;
use game_data::game_actions::{GameAction, GameStateAction};
use game_data::game_state::{GameConfiguration, GameState, MulliganDecision};
use game_data::player_name::{AIPlayer, PlayerId};
use game_data::utils;
use once_cell::sync::Lazy;
use panel_address::Panel;
use player_data::PlayerStatus;
use protos::riftcaller::client_debug_command::DebugCommand;
use protos::riftcaller::game_command::Command;
use protos::riftcaller::{ClientAction, ClientDebugCommand, LoadSceneCommand, SceneLoadMode};
use rules::mutations::SummonMinion;
use rules::{curses, dispatch, draw_cards, mana, mutations, wounds};
use serde_json::{de, ser};
use sled::Db;
use ulid::Ulid;
use user_action_data::{
    DebugAction, DebugScenario, NamedDeck, NewGameAction, NewGameDebugOptions, NewGameDeck,
    UserAction,
};
use with_error::WithError;

use crate::server_data::{ClientData, GameResponse, RequestData};
use crate::{adventure_server, requests};

static DEBUG_DB: Lazy<Db> = Lazy::new(|| sled::open("debug_db").expect("Error opening debug_db"));

fn get_current_game_id() -> Result<GameId> {
    DEBUG_DB
        .open_tree("debug")
        .with_error(|| "Error opening debug tree")?
        .get("debug_game_id")
        .with_error(|| "Error fetching debug game id")?
        .map(|slice| de::from_slice::<GameId>(&slice).with_error(|| "Error deserializing game id"))
        .with_error(|| "Error fetching debug game id")?
}

fn set_current_game_id(game_id: GameId) -> Result<()> {
    DEBUG_DB.open_tree("debug").with_error(|| "Error opening debug tree")?.insert(
        "debug_game_id",
        ser::to_vec(&game_id).with_error(|| "Error serializing game id")?,
    )?;
    DEBUG_DB.flush()?;
    Ok(())
}

const DEBUG_ABILITY_ID: AbilityId = AbilityId {
    card_id: CardId { side: Side::Overlord, index: usize::MAX },
    index: AbilityIndex(0),
};

pub async fn handle_debug_action(
    database: &impl Database,
    data: &RequestData,
    action: &DebugAction,
    request_fields: &HashMap<String, String>,
) -> Result<GameResponse> {
    match action {
        DebugAction::NewGame(side) => create_debug_game(data, *side),
        DebugAction::JoinGame(side) => {
            let game_id = get_current_game_id()?;
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
                mutations::score_bonus_points(game, user_side, *amount)?;
                Ok(())
            })
            .await
        }
        DebugAction::SaveGameState(index) => {
            let current_id = data.game_id.with_error(|| "Expected active game")?;
            let mut game = requests::fetch_game(database, data.game_id).await?;
            let game_id = GameId::new_from_u128(100 + index);
            game.id = game_id;
            database.write_game(&game).await?;
            set_current_game_id(current_id)?;
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
                curses::give_curses(game, DEBUG_ABILITY_ID, *amount)?;
                Ok(())
            })
            .await
        }
        DebugAction::RemoveCurses(amount) => {
            debug_update_game(database, data, |game, _| {
                curses::remove_curses(game, *amount)?;
                Ok(())
            })
            .await
        }
        DebugAction::AddWounds(amount) => {
            debug_update_game(database, data, |game, _| {
                wounds::give(game, DEBUG_ABILITY_ID, *amount)?;
                Ok(())
            })
            .await
        }
        DebugAction::RemoveWounds(amount) => {
            debug_update_game(database, data, |game, _| {
                wounds::remove_wounds(game, *amount)?;
                Ok(())
            })
            .await
        }
        DebugAction::FilterCardList { position, metadata, turn_face_up } => {
            let input = request_fields.get("CardVariant").with_error(|| "Expected CardVariant")?;
            Ok(GameResponse::new(ClientData::propagate(data)).command(panels::update(
                AddToZonePanel::new(input, *position, *metadata, *turn_face_up)
                    .build_panel()
                    .unwrap(),
            )))
        }
        DebugAction::AddToZone { variant, position, turn_face_up } => {
            debug_update_game(database, data, |game, _| {
                let side = rules::get(*variant).side;
                if let Some(card_id) = mutations::realize_top_of_deck(game, side, 1)?.get(0) {
                    mutations::overwrite_card(game, *card_id, *variant)?;
                    mutations::set_visible_to(game, *card_id, card_id.side, true);

                    if matches!(position, CardPosition::Hand(s) if *s == side) {
                        draw_cards::run(game, side, 1, InitiatedBy::GameAction)?;
                    } else if matches!(position, CardPosition::DiscardPile(_)) {
                        mutations::discard_card(game, *card_id)?;
                    } else {
                        mutations::move_card(game, *card_id, *position)?;
                    }

                    if *turn_face_up {
                        mutations::turn_face_up(game, *card_id);
                    }
                }
                Ok(())
            })
            .await
        }
        DebugAction::ApplyScenario(scenario) => {
            let game = apply_scenario(*scenario, data)?;
            let game_id = game.id;

            let mut player = requests::fetch_player(database, data.player_id).await?;
            if data.player_id == game.overlord.id {
                player.status = Some(PlayerStatus::Playing(game.id, Side::Overlord));
                database.write_player(&player).await?;
            } else {
                player.status = Some(PlayerStatus::Playing(game.id, Side::Champion));
                database.write_player(&player).await?;
            }

            database.write_game(&game).await?;
            set_current_game_id(game_id)?;

            reload_scene(
                &RequestData {
                    player_id: data.player_id,
                    game_id: Some(game_id),
                    adventure_id: data.adventure_id,
                },
                &game,
            )
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
    set_current_game_id(id)?;
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

fn apply_scenario(scenario: DebugScenario, data: &RequestData) -> Result<GameState> {
    let id = GameId::generate();
    let mut game = scenario_game(id, data.player_id, scenario.side())?;

    match scenario {
        DebugScenario::NewGameOverlord => {}
        DebugScenario::NewGameChampion => {}
        DebugScenario::VsInfernalMinionAndScheme => {
            vs_minion_and_scheme(&mut game, CardName::TestInfernalMinion)?;
        }
        DebugScenario::VsAstralMinionAndScheme => {
            vs_minion_and_scheme(&mut game, CardName::TestAstralMinion)?;
        }
        DebugScenario::VsMortalMinionAndScheme => {
            vs_minion_and_scheme(&mut game, CardName::TestMortalMinion)?;
        }
    }

    Ok(game)
}

fn vs_minion_and_scheme(game: &mut GameState, minion: CardName) -> Result<()> {
    create_at_position(
        game,
        CardName::TestScheme3_10,
        CardPosition::Room(
            CardPlayId(utils::DEBUG_EVENT_ID.fetch_add(1, Ordering::Relaxed)),
            RoomId::RoomA,
            RoomLocation::Occupant,
        ),
    )?;
    let minion_id = create_at_position(
        game,
        minion,
        CardPosition::Room(
            CardPlayId(utils::DEBUG_EVENT_ID.fetch_add(1, Ordering::Relaxed)),
            RoomId::RoomA,
            RoomLocation::Defender,
        ),
    )?;
    mutations::summon_minion(game, minion_id, InitiatedBy::GameAction, SummonMinion::IgnoreCosts)
}

fn scenario_game(game_id: GameId, player_id: PlayerId, side: Side) -> Result<GameState> {
    let mut result = GameState::new(
        game_id,
        if side == Side::Overlord { player_id } else { PlayerId::AI(AIPlayer::NoAction) },
        decklists::CANONICAL_OVERLORD.clone(),
        if side == Side::Champion { player_id } else { PlayerId::AI(AIPlayer::NoAction) },
        decklists::CANONICAL_CHAMPION.clone(),
        GameConfiguration::default(),
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
    mutations::set_visible_to(game, card_id, card_id.side, true);
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

        let user_result = render::render_updates(game, user_side, None)?;
        let opponent_id = game.player(user_side.opponent()).id;
        let opponent_commands = render::render_updates(game, user_side.opponent(), None)?;

        Ok(GameResponse::new(ClientData::with_game_id(data, Some(game.id)))
            .commands(user_result)
            .opponent_response(opponent_id, opponent_commands))
    })
    .await
}
