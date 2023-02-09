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

use game_data::game::{GameState, HistoryEvent, TurnData};
use game_data::primitives::{CardId, RoomId};

/// Returns the record of game events which happened on a given `turn`.
pub fn for_turn(game: &GameState, turn: TurnData) -> impl Iterator<Item = HistoryEvent> + '_ {
    game.history.iter().filter_map(move |entry| (entry.turn == turn).then_some(entry.event))
}

/// Returns the record of game events which happened on the current
/// player's turn so far.
pub fn current_turn(game: &GameState) -> impl Iterator<Item = HistoryEvent> + '_ {
    for_turn(game, game.data.turn)
}

/// Returns an iterator over cards which have been played in the current
/// player's turn so far.
pub fn cards_played_this_turn(game: &GameState) -> impl Iterator<Item = CardId> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::PlayedCard(id) = h {
            Some(id)
        } else {
            None
        }
    })
}

/// Returns an iterator over rooms which have been raided in the current
/// player's turn so far.
pub fn rooms_raided_this_turn(game: &GameState) -> impl Iterator<Item = RoomId> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::RaidBegan(room_id) = h {
            Some(room_id)
        } else {
            None
        }
    })
}

/// Returns an iterator over rooms which have been successfully raided in the
/// current player's turn so far.
pub fn raid_accesses_this_turn(game: &GameState) -> impl Iterator<Item = RoomId> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::RaidSuccess(room_id) = h {
            Some(room_id)
        } else {
            None
        }
    })
}
