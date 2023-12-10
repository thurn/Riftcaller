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

use game_data::game_actions::ButtonPromptContext;
use game_data::prompt_data::{ButtonPrompt, GamePrompt, PromptChoice, RoomSelectorPrompt};

/// Adds a choice prompt for the `side` player containing the choices in
/// `choices`.
pub fn with_context_and_choices(
    context: ButtonPromptContext,
    choices: Vec<PromptChoice>,
) -> Option<GamePrompt> {
    Some(GamePrompt::ButtonPrompt(ButtonPrompt { context: Some(context), choices }))
}

/// Adds a choice prompt for the `side` player containing the choices in
/// `choices`.
pub fn with_choices(choices: Vec<PromptChoice>) -> Option<GamePrompt> {
    Some(GamePrompt::ButtonPrompt(ButtonPrompt { context: None, choices }))
}

/// Show a priority window prompt if one is not already displayed. This prompt
/// allows a player to activate abilities when they otherwise could not.
pub fn priority_window() -> Option<GamePrompt> {
    Some(GamePrompt::PriorityPrompt)
}

/// Show a room selector prompt to a player.
///
/// Has no effect if the `valid_rooms` on the provided prompt is empty.
pub fn room_selector(prompt: RoomSelectorPrompt) -> Option<GamePrompt> {
    if prompt.valid_rooms.is_empty() {
        return None;
    }

    Some(GamePrompt::RoomSelector(prompt))
}
