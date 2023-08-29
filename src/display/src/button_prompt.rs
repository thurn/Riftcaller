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
use game_data::game_actions::{ButtonPrompt, PromptContext};
use game_data::primitives::Side;
use prompts::effect_prompts;
use prompts::game_instructions::GameInstructions;
use prompts::prompt_container::PromptContainer;
use protos::spelldawn::InterfaceMainControls;

pub fn controls(user_side: Side, prompt: &ButtonPrompt) -> Option<InterfaceMainControls> {
    let context = match prompt.context {
        Some(PromptContext::MinionRoomLimit(_)) => GameInstructions::new(
            "Minion limit exceeded. You must sacrifice a minion in this room.".to_string(),
        )
        .build(),
        _ => None,
    };

    let mut main_controls: Vec<Box<dyn ComponentObject>> = vec![];
    let mut card_anchor_nodes = vec![];

    for (i, choice) in prompt.choices.iter().enumerate() {
        let button = effect_prompts::button(user_side, i, choice);
        if button.has_anchor() {
            card_anchor_nodes.push(button.render_to_card_anchor_node());
        } else {
            main_controls.push(Box::new(button));
        }
    }

    Some(InterfaceMainControls {
        node: PromptContainer::new().children(main_controls).build(),
        overlay: context,
        card_anchor_nodes,
    })
}
