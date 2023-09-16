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
use game_data::game_actions::{GameAction, PlayCardBrowser, PromptAction, PromptContext};
use prompts::game_instructions::GameInstructions;
use prompts::prompt_container::PromptContainer;
use prompts::response_button::ResponseButton;
use protos::spelldawn::InterfaceMainControls;

pub fn controls(prompt: &PlayCardBrowser) -> Option<InterfaceMainControls> {
    match prompt.context {
        Some(PromptContext::PlayFromDiscard(card_type)) => Some(InterfaceMainControls {
            node: buttons().build(),
            overlay: GameInstructions::new(format!(
                "Play {} {} from your discard pile.",
                card_type.article(),
                card_type.to_string().to_lowercase()
            ))
            .build(),
            card_anchor_nodes: vec![],
        }),
        _ => None,
    }
}

fn buttons() -> PromptContainer {
    PromptContainer::new().child(
        ResponseButton::new("Skip")
            .primary(false)
            .action(GameAction::PromptAction(PromptAction::SkipPlayingCard)),
    )
}
