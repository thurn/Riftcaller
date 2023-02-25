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
use database::Database;
use game_data::deck::Deck;
use game_data::game::{GameConfiguration, GameState};
use game_data::player_name::{NamedPlayer, PlayerId};
use game_data::primitives::Side;
use player_data::{PlayerData, PlayerStatus};
use rules::{dispatch, mutations};
use tracing::info;
use tutorial::tutorial_actions;
use user_action_data::{NewGameAction, NewGameDeck};
use with_error::fail;

use crate::ai_agent_response::IncrementalUpdates;
use crate::server_data::{ClientData, GameResponse, OpponentData, RequestData};
use crate::{ai_agent_response, requests};

/// Creates a new game and assigns the player to their requested side & deck.
pub async fn create(
    database: &impl Database,
    data: &RequestData,
    action: &NewGameAction,
) -> Result<GameResponse> {
    let (mut player, opponent) = tokio::try_join!(
        requests::fetch_player(database, data.player_id),
        find_opponent(database, action.opponent)
    )?;

    let opponent_id = action.opponent;
    let debug_options = action.debug_options.unwrap_or_default();
    let user_deck = find_deck(&player, action.deck)?;
    let opponent_deck = if let Some(deck) = requested_deck(&opponent, user_deck.side.opponent())? {
        deck
    } else {
        // TODO: Implement UI for this this
        player.status = Some(PlayerStatus::RequestedGame(*action));
        database.write_player(&player).await?;
        return Ok(GameResponse::new(ClientData::propagate(data)));
    };

    let (user_side, opponent_side) = (user_deck.side, opponent_deck.side);
    let (overlord_deck, champion_deck, overlord_id, champion_id) = match (user_side, opponent_side)
    {
        (Side::Overlord, Side::Champion) => (user_deck, opponent_deck, player.id, opponent_id),
        (Side::Champion, Side::Overlord) => (opponent_deck, user_deck, opponent_id, player.id),
        _ => fail!("Deck side mismatch!"),
    };

    let game_id = if let Some(id) = debug_options.override_game_id {
        id
    } else {
        database.generate_game_id()
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

    player.status = Some(PlayerStatus::Playing(game_id));

    // Handle mulligan decision if AI is first to act.
    ai_agent_response::maybe_run_ai(data, &mut game, IncrementalUpdates::Skip).await?;

    let result = Ok(GameResponse::new(ClientData::with_game_id(data, Some(game_id)))
        .command(requests::force_load_scene("Game"))
        .opponent_response(opponent_id, vec![requests::force_load_scene("Game")]));

    database.write_game(&game).await?;
    database.write_player(&player).await?;
    if let OpponentData::HumanPlayer(mut o) = opponent {
        o.status = Some(PlayerStatus::Playing(game_id));
        database.write_player(&o).await?;
    }

    result
}

async fn find_opponent(database: &impl Database, opponent_id: PlayerId) -> Result<OpponentData> {
    match opponent_id {
        PlayerId::Database(_) => {
            let opponent = requests::fetch_player(database, opponent_id).await?;
            Ok(OpponentData::HumanPlayer(Box::new(opponent)))
        }
        PlayerId::Named(name) => Ok(OpponentData::NamedPlayer(name)),
    }
}

fn requested_deck(opponent: &OpponentData, side: Side) -> Result<Option<Deck>> {
    Ok(match opponent {
        OpponentData::HumanPlayer(player) => match player.status {
            Some(PlayerStatus::RequestedGame(action)) => Some(find_deck(player, action.deck)?),
            _ => None,
        },
        // TODO: Each named player should have their own decklist
        OpponentData::NamedPlayer(name) => match name {
            NamedPlayer::TutorialOpponent => Some(decklists::TUTORIAL_OVERLORD.clone()),
            NamedPlayer::DebugChampion => Some(decklists::CANONICAL_CHAMPION.clone()),
            NamedPlayer::DebugOverlord => Some(decklists::CANONICAL_OVERLORD.clone()),
            _ => Some(decklists::basic_deck(side)),
        },
    })
}

fn find_deck(player: &PlayerData, deck: NewGameDeck) -> Result<Deck> {
    Ok(match deck {
        NewGameDeck::DeckId(id) => player.deck(id)?.clone(),
        NewGameDeck::NamedDeck(name) => decklists::named_deck(name),
    })
}
