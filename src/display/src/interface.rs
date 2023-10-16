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

use game_data::game_actions::{ActionButtons, GamePrompt, GameStateAction};
use game_data::game_state::{GamePhase, GameState, MulliganDecision};
use game_data::primitives::Side;
use prompts::prompts;
use protos::spelldawn::InterfaceMainControls;
use raid_state::raid_prompt;
use rules::flags;

use crate::{button_prompt, card_selector, play_card_browser};

/// Returns a [InterfaceMainControls] to render the interface state for the
/// provided `game`.
pub fn render(game: &GameState, side: Side) -> Option<InterfaceMainControls> {
    let current_prompt = &game.player(side).prompt_queue.get(0);
    if let Some(current) = current_prompt {
        return match current {
            GamePrompt::ButtonPrompt(prompt) => button_prompt::controls(game, side, prompt),
            GamePrompt::CardSelector(prompt) => card_selector::controls(prompt),
            GamePrompt::PlayCardBrowser(prompt) => play_card_browser::controls(prompt),
        };
    } else if let Some(raid) = &game.raid {
        return raid_prompt::build(game, raid, side);
    } else if let GamePhase::ResolveMulligans(_) = &game.info.phase {
        if flags::can_make_mulligan_decision(game, side) {
            return prompts::action_prompt(&ActionButtons {
                context: None,
                responses: vec![
                    GameStateAction::MulliganDecision(MulliganDecision::Keep),
                    GameStateAction::MulliganDecision(MulliganDecision::Mulligan),
                ],
            });
        }
    } else if flags::can_take_start_turn_action(game, side) {
        return prompts::action_prompt(&ActionButtons {
            context: None,
            responses: vec![GameStateAction::StartTurnAction],
        });
    } else if flags::can_take_end_turn_action(game, side) {
        return prompts::action_prompt(&ActionButtons {
            context: None,
            responses: vec![GameStateAction::EndTurnAction],
        });
    }

    None
}
