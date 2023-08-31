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

use core_ui::action_builder::ActionBuilder;
use game_data::game::MulliganDecision;
use game_data::game_actions::{GameAction, GameStateAction};

use crate::response_button::ResponseButton;

pub fn for_prompt(action: GameStateAction) -> ResponseButton {
    match action {
        GameStateAction::MulliganDecision(data) => mulligan_button(data),
        GameStateAction::StartTurnAction => ResponseButton::new("Start Turn"),
        GameStateAction::EndTurnAction => ResponseButton::new("End Turn"),
    }
    .action(ActionBuilder::new().action(GameAction::GameStateAction(action)).build())
}

fn mulligan_button(mulligan: MulliganDecision) -> ResponseButton {
    match mulligan {
        MulliganDecision::Keep => ResponseButton::new("Keep"),
        MulliganDecision::Mulligan => ResponseButton::new("Mulligan").primary(false),
    }
}
