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

use game_data::game::{GamePhase, GameState, TurnState};
use game_data::primitives::{CardId, Side};
use raids::RaidDataExt;
use rules::flags;

/// Returns the player that is currently able to take actions in the provided
/// game. If no player can act because the game has ended, returns None.
pub fn current_priority(game: &GameState) -> Option<Side> {
    match &game.info.phase {
        GamePhase::ResolveMulligans(_) => {
            if flags::can_make_mulligan_decision(game, Side::Overlord) {
                Some(Side::Overlord)
            } else {
                assert!(
                    flags::can_make_mulligan_decision(game, Side::Champion),
                    "No player has mulligan decision"
                );
                Some(Side::Champion)
            }
        }
        GamePhase::Play => {
            if !game.overlord.card_prompt_queue.is_empty() {
                Some(Side::Overlord)
            } else if !game.champion.card_prompt_queue.is_empty() {
                Some(Side::Champion)
            } else if let Some(raid) = &game.info.raid {
                Some(raid.phase().active_side())
            } else if game.info.turn_state == TurnState::Active {
                Some(game.info.turn.side)
            } else {
                Some(game.info.turn.side.opponent())
            }
        }
        GamePhase::GameOver { .. } => None,
    }
}

/// Returns true if the `side` player currently has priority as described by
/// [current_priority].
pub fn has_priority(game: &GameState, side: Side) -> bool {
    current_priority(game) == Some(side)
}

/// Is the `side` player currently able to unveil the provided card?
pub fn can_take_unveil_card_action(game: &GameState, side: Side, card_id: CardId) -> bool {
    flags::can_take_game_actions(game)
        && side == Side::Overlord
        && has_priority(game, side)
        && game.card(card_id).is_face_down()
        && game.card(card_id).position().in_play()
        && flags::can_pay_card_cost(game, card_id)
}
