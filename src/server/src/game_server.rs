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

use adapters::ServerCardId;
use anyhow::Result;
use database::Database;
use display::render;
use game_data::game::GameState;
use game_data::game_actions::{self, GameAction};
use game_data::primitives::{GameId, Side};
use player_data::PlayerData;
use protos::spelldawn::{
    DrawCardAction, GainManaAction, InitiateRaidAction, LevelUpRoomAction, PlayCardAction,
    SpendActionPointAction,
};
use tracing::info;
use tutorial::tutorial_actions;
use with_error::WithError;

use crate::ai_agent_response::IncrementalUpdates;
use crate::server_data::{ClientData, GameResponse, RequestData};
use crate::{ai_agent_response, requests};

pub async fn connect(
    database: &impl Database,
    player: &PlayerData,
    game_id: GameId,
) -> Result<GameResponse> {
    let game = requests::fetch_game(database, Some(game_id)).await?;
    info!(?player.id, ?game.id, "Connected to game");
    let side = game.player_side(player.id)?;
    let mut commands = vec![requests::load_scene("Game")];
    commands.append(&mut render::connect(&game, side)?);
    let client_data = ClientData {
        adventure_id: player.adventure.as_ref().map(|a| a.id),
        game_id: Some(game.id),
    };
    let mut result = GameResponse::new(client_data).commands(commands);
    requests::add_panels(database, player.id, None, &mut result).await?;
    Ok(result)
}

pub async fn handle_leave_game(
    database: &mut impl Database,
    data: &RequestData,
) -> Result<GameResponse> {
    requests::with_player(database, data, |player| {
        player.status = None;
        Ok(GameResponse::new(ClientData::with_game_id(data, None))
            .command(requests::load_scene("Main")))
    })
    .await
}

pub async fn handle_game_action(
    database: &mut impl Database,
    data: &RequestData,
    action: &GameAction,
) -> Result<GameResponse> {
    let mut game = requests::fetch_game(database, data.game_id).await?;
    let user_side = game.player_side(data.player_id)?;
    apply_game_action(&mut game, user_side, action)?;

    let ran_agent =
        ai_agent_response::maybe_run_ai(data, &mut game, IncrementalUpdates::Send).await?;

    let mut result = if ran_agent {
        // In order to avoid a race between incremental updates and the server
        // response, we send an empty response when an AI opponent is playing.
        GameResponse::new(ClientData::with_game_id(data, Some(game.id)))
    } else {
        let user_result = render::render_updates(&game, user_side)?;
        let opponent_id = game.player(user_side.opponent()).id;
        let opponent_commands = render::render_updates(&game, user_side.opponent())?;

        GameResponse::new(ClientData::with_game_id(data, Some(game.id)))
            .commands(user_result)
            .opponent_response(opponent_id, opponent_commands)
    };

    requests::add_panels(database, data.player_id, None, &mut result).await?;
    database.write_game(&game).await?;
    Ok(result)
}

pub fn apply_game_action(game: &mut GameState, side: Side, action: &GameAction) -> Result<()> {
    tutorial_actions::handle_game_action(game, Some(action))?;
    actions::handle_game_action(game, side, action)?;
    tutorial_actions::handle_game_action(game, None)?;
    Ok(())
}

pub async fn handle_gain_mana(
    database: &mut impl Database,
    data: &RequestData,
    _: &GainManaAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Gain Mana");
    handle_game_action(database, data, &GameAction::GainMana).await
}

pub async fn handle_draw_card(
    database: &mut impl Database,
    data: &RequestData,
    _: &DrawCardAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Draw Card");
    handle_game_action(database, data, &GameAction::DrawCard).await
}

pub async fn handle_play_card(
    database: &mut impl Database,
    data: &RequestData,
    action: &PlayCardAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Play Card");
    let action = match adapters::server_card_id(action.card_id.with_error(|| "CardID expected")?)? {
        ServerCardId::CardId(card_id) => GameAction::PlayCard(card_id, card_target(&action.target)),
        ServerCardId::AbilityId(ability_id) => {
            GameAction::ActivateAbility(ability_id, card_target(&action.target))
        }
    };
    handle_game_action(database, data, &action).await
}

pub async fn handle_level_up_room(
    database: &mut impl Database,
    data: &RequestData,
    action: &LevelUpRoomAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Level Up Room");
    let room_id = adapters::room_id(action.room_id)?;
    handle_game_action(database, data, &GameAction::LevelUpRoom(room_id)).await
}

pub async fn handle_initiate_raid(
    database: &mut impl Database,
    data: &RequestData,
    action: &InitiateRaidAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Initiate Raid");
    let room_id = adapters::room_id(action.room_id)?;
    handle_game_action(database, data, &GameAction::InitiateRaid(room_id)).await
}

pub async fn handle_spend_action_point(
    database: &mut impl Database,
    data: &RequestData,
    _: &SpendActionPointAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Spend Action Point");
    handle_game_action(database, data, &GameAction::SpendActionPoint).await
}

/// Applies a game mutation and produces a snapshot of the resulting game state
/// to send to both players.
pub async fn update_game(
    database: &mut impl Database,
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

fn card_target(target: &Option<protos::spelldawn::CardTarget>) -> game_actions::CardTarget {
    target.as_ref().map_or(game_actions::CardTarget::None, |t| {
        t.card_target.as_ref().map_or(game_actions::CardTarget::None, |t2| match t2 {
            protos::spelldawn::card_target::CardTarget::RoomId(room_id) => {
                adapters::room_id(*room_id)
                    .map_or(game_actions::CardTarget::None, game_actions::CardTarget::Room)
            }
        })
    })
}
