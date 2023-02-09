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

use game_data::game::{GameState, HistoryAction, TurnData};
use game_data::game_actions::GameAction;
use game_data::primitives::CardId;

/// Returns the record of game actions which happened on a given `turn`.
pub fn for_turn(game: &GameState, turn: TurnData) -> impl Iterator<Item = &HistoryAction> {
    game.history.iter().filter(move |a| a.turn == turn)
}

/// Returns the record of game actions which happened on the current
/// player's turn so far.
pub fn current_turn(game: &GameState) -> impl Iterator<Item = &HistoryAction> {
    let current = game.data.turn;
    game.history.iter().filter(move |a| a.turn == current)
}

/// Returns an iterator over cards which have been played in the current
/// player's turn so far.
pub fn cards_played_this_turn(game: &GameState) -> impl Iterator<Item = CardId> + '_ {
    current_turn(game).filter_map(move |h| {
        if let GameAction::PlayCard(id, _) = h.action {
            Some(id)
        } else {
            None
        }
    })
}
