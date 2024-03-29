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

use adapters::ServerCardId;
use adventure_data::adventure::AdventureScreen;
use anyhow::Result;
use core_data::game_primitives::{GameId, Side};
use database::Database;
use display::{render, set_display_preference};
use game_data::game_actions::{self, DisplayPreference, GameAction};
use game_data::game_state::GameState;
use game_data::prompt_data::PromptAction;
use player_data::PlayerState;
use protos::riftcaller::{
    DrawCardAction, GainManaAction, InitiateRaidAction, MoveCardAction, PlayCardAction,
    ProgressRoomAction, SpendActionPointAction,
};
use tracing::{debug, info};
use tutorial::tutorial_actions;
use user_action_data::GameOutcome;
use with_error::{fail, WithError};

use crate::ai_agent_response::IncrementalUpdates;
use crate::requests::SceneName;
use crate::server_data::{ClientData, GameResponse, RequestData};
use crate::{ai_agent_response, requests};

pub async fn connect(
    database: &impl Database,
    player: &PlayerState,
    game_id: GameId,
) -> Result<GameResponse> {
    let game = requests::fetch_game(database, Some(game_id)).await?;
    info!(?player.id, ?game.id, "Connected to game");
    let side = game.player_side(player.id)?;
    let mut commands = vec![requests::load_scene(SceneName::Game)];
    commands.append(&mut render::connect(&game, side)?);
    let client_data = ClientData {
        adventure_id: player.adventure.as_ref().map(|a| a.id),
        game_id: Some(game.id),
    };
    let mut result = GameResponse::new(client_data).commands(commands);
    requests::add_standard_ui(
        &mut result,
        player,
        Some(&game),
        set_display_preference::button(&game, side, None),
    )
    .await?;
    Ok(result)
}

pub async fn handle_leave_game(
    database: &impl Database,
    data: &RequestData,
    outcome: GameOutcome,
) -> Result<GameResponse> {
    requests::with_player(database, data, |player| {
        player.status = None;
        let scene = if let Some(adventure) = player.adventure.as_mut() {
            let Some(AdventureScreen::Battle(battle)) = adventure.screens.current() else {
                fail!("Expected player to be in a battle")
            };

            adventure.coins += battle.reward;
            adventure.screens.pop();

            match outcome {
                GameOutcome::Victory => SceneName::World,
                GameOutcome::Defeat => SceneName::Main,
            }
        } else {
            SceneName::Main
        };

        Ok(GameResponse::new(ClientData::with_game_id(data, None))
            .command(requests::load_scene(scene)))
    })
    .await
}

pub async fn handle_game_action(
    database: &impl Database,
    data: &RequestData,
    action: &GameAction,
) -> Result<GameResponse> {
    let mut game = requests::fetch_game(database, data.game_id).await?;
    let user_side = game.player_side(data.player_id)?;
    apply_game_action(&mut game, user_side, action)?;

    let ran_agent =
        ai_agent_response::maybe_run_ai(data, &mut game, IncrementalUpdates::Send).await?;

    let result = if ran_agent {
        debug!("AI connected, sending empty response");
        // In order to avoid a race between incremental updates and the server
        // response, we send an empty response when an AI opponent is playing.
        GameResponse::new(ClientData::with_game_id(data, Some(game.id)))
    } else {
        let display_preference = display_preference_for_action(action);
        let user_result = render::render_updates(&game, user_side, display_preference)?;
        let opponent_id = game.player(user_side.opponent()).id;
        let opponent_commands = render::render_updates(&game, user_side.opponent(), None)?;

        let mut result = GameResponse::new(ClientData::with_game_id(data, Some(game.id)))
            .commands(user_result)
            .opponent_response(opponent_id, opponent_commands);
        let player = requests::fetch_player(database, data.player_id).await?;
        requests::add_standard_ui(
            &mut result,
            &player,
            Some(&game),
            set_display_preference::button(&game, user_side, display_preference),
        )
        .await?;
        result
    };

    database.write_game(&game).await?;
    Ok(result)
}

fn display_preference_for_action(action: &GameAction) -> Option<DisplayPreference> {
    match action {
        GameAction::SetDisplayPreference(p) => Some(*p),
        _ => None,
    }
}

pub fn apply_game_action(game: &mut GameState, side: Side, action: &GameAction) -> Result<()> {
    tutorial_actions::handle_game_action(game, Some(action))?;
    actions::handle_game_action(game, side, action)?;
    tutorial_actions::handle_game_action(game, None)?;
    Ok(())
}

pub async fn handle_gain_mana(
    database: &impl Database,
    data: &RequestData,
    _: &GainManaAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Gain Mana");
    handle_game_action(database, data, &GameAction::GainMana).await
}

pub async fn handle_draw_card(
    database: &impl Database,
    data: &RequestData,
    _: &DrawCardAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Draw Card");
    handle_game_action(database, data, &GameAction::DrawCard).await
}

pub async fn handle_play_card(
    database: &impl Database,
    data: &RequestData,
    action: &PlayCardAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Play Card");
    let action = match adapters::server_card_id(action.card_id.with_error(|| "CardID expected")?)? {
        ServerCardId::CardId(card_id) => GameAction::PlayCard(card_id, card_target(&action.target)),
        ServerCardId::AbilityId(ability_id) => {
            GameAction::ActivateAbility(ability_id, card_target(&action.target))
        }
        ServerCardId::SummonProject(card_id) => GameAction::SummonProject(card_id),
        ServerCardId::CurseCard => GameAction::RemoveCurse,
        ServerCardId::DispelCard => GameAction::DispelEvocation,
        ServerCardId::RoomSelectorCard => GameAction::PromptAction(PromptAction::RoomPromptSelect(
            card_target(&action.target).room_id()?,
        )),
    };
    handle_game_action(database, data, &action).await
}

pub async fn handle_progress_room(
    database: &impl Database,
    data: &RequestData,
    action: &ProgressRoomAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Progress Room");
    let room_id = adapters::room_id(action.room_id)?;
    handle_game_action(database, data, &GameAction::ProgressRoom(room_id)).await
}

pub async fn handle_initiate_raid(
    database: &impl Database,
    data: &RequestData,
    action: &InitiateRaidAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Initiate Raid");
    let room_id = adapters::room_id(action.room_id)?;
    handle_game_action(database, data, &GameAction::InitiateRaid(room_id)).await
}

pub async fn handle_spend_action_point(
    database: &impl Database,
    data: &RequestData,
    _: &SpendActionPointAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Spend Action Point");
    handle_game_action(database, data, &GameAction::SpendActionPoint).await
}

pub async fn handle_move_card(
    database: &impl Database,
    data: &RequestData,
    action: &MoveCardAction,
) -> Result<GameResponse> {
    info!(?data.player_id, "Move Card");
    let card_id = match adapters::server_card_id(action.card_id.with_error(|| "CardID expected")?)?
    {
        ServerCardId::CardId(card_id) => card_id,
        _ => fail!("Expected standard CardId"),
    };
    handle_game_action(
        database,
        data,
        &GameAction::MoveSelectorCard { card_id, index: action.index },
    )
    .await
}

fn card_target(target: &Option<protos::riftcaller::CardTarget>) -> game_actions::CardTarget {
    target.as_ref().map_or(game_actions::CardTarget::None, |t| {
        t.card_target.as_ref().map_or(game_actions::CardTarget::None, |t2| match t2 {
            protos::riftcaller::card_target::CardTarget::RoomId(room_id) => {
                adapters::room_id(*room_id)
                    .map_or(game_actions::CardTarget::None, game_actions::CardTarget::Room)
            }
        })
    })
}
