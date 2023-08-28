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

use core_ui::prelude::*;
use game_data::game_actions::{CardButtonPrompt, PromptAction, PromptContext};
use game_data::primitives::Side;
use prompts::action_buttons;
use prompts::game_instructions::GameInstructions;
use protos::spelldawn::InterfaceMainControls;

pub fn controls(user_side: Side, prompt: &CardButtonPrompt) -> Option<InterfaceMainControls> {
    let context = match prompt.context {
        Some(PromptContext::MinionRoomLimit(_)) => GameInstructions::new(
            "Minion limit exceeded, you must sacrifice a minion in this room.".to_string(),
        )
        .build(),
        _ => None,
    };

    Some(InterfaceMainControls {
        node: None,
        overlay: context,
        card_anchor_nodes: prompt
            .choices
            .iter()
            .map(|choice| {
                action_buttons::card_response_button(user_side, choice.action)
                    .action(PromptAction::CardAction(choice.action))
                    .render_to_card_anchor_node()
            })
            .collect(),
    })
}
