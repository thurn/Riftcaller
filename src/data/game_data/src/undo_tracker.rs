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

use serde::{Deserialize, Serialize};

use crate::game_state::GameState;
use crate::primitives::{CardId, Side};

/// State for the undo system.
///
/// This system allows the player to undo & redo their moves by storing
/// snapshots of the game state. It also tracks whether the current game state
/// step was reached by employing randomness or revealing a hidden card, since
/// undoing that kind of operation would effectively be cheating at the game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UndoTracker {
    /// Previous state to jump to as a result of an 'undo' operation, if any.
    pub undo: Option<Box<GameState>>,
    /// Set to true if the current game state invoked the random number
    /// generator at any time during its action resolution process.
    pub random: bool,
    /// Set to true if the current game state revealed any face-down cards as
    /// part of its action resolution process.
    pub visible: bool,
    /// Player who acted to produce this game state, or None in the starting
    /// state of a game.
    pub side: Option<Side>,
}

/// Updates the `visible` state of the provided game's [UndoTracker] if
/// invoking the `action` function caused the `card_id` card to transition from
/// not-revealed to revealed for the active player.
pub fn track_visible_state(
    game: &mut GameState,
    card_id: CardId,
    action: impl Fn(&mut GameState) -> (),
) {
    if let Some(side) = &game.undo_tracker.as_ref().and_then(|u| u.side) {
        let previously_revealed = game.card(card_id).is_visible_to(*side);
        action(game);
        if !previously_revealed && game.card(card_id).is_visible_to(*side) {
            if let Some(undo_tracker) = &mut game.undo_tracker {
                undo_tracker.visible = true;
            }
        }
    } else {
        action(game)
    }
}
