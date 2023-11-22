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

use adapters::response_builder::{ResponseBuilder, ResponseState};
use core_data::game_primitives::Side;
use game_data::game_actions::DisplayPreference;
use game_data::game_state::{GamePhase, GameState};

use crate::{positions, sync};

/// Returns a [DisplayPreference] which can be requested in the current game
/// state, via the "set display preference" button, if applicable.
///
/// Example: A button to hide the raid browser and view the arena.
pub fn button(
    game: &GameState,
    user_side: Side,
    current: Option<DisplayPreference>,
) -> Option<DisplayPreference> {
    if game.info.phase != GamePhase::Play {
        return None;
    }

    if Some(DisplayPreference::ShowArenaView(true)) == current {
        return Some(DisplayPreference::ShowArenaView(false));
    }

    let builder = ResponseBuilder::new(
        user_side,
        ResponseState { animate: false, is_final_update: true, display_preference: None },
    );
    if game
        .all_cards()
        .filter(|c| !sync::skip_sending_to_client(c))
        .any(|c| positions::has_position_override(&builder, game, c))
    {
        Some(DisplayPreference::ShowArenaView(true))
    } else {
        None
    }
}
