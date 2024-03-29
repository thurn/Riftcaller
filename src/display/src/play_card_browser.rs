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

use core_ui::prelude::*;
use game_data::game_actions::GameAction;
use game_data::prompt_data::{PlayCardBrowser, PromptAction, PromptContext};
use prompt_ui::game_instructions::GameInstructions;
use prompt_ui::prompt_container::PromptContainer;
use prompt_ui::response_button::ResponseButton;
use protos::riftcaller::InterfaceMainControls;

pub fn controls(prompt: &PlayCardBrowser) -> Option<InterfaceMainControls> {
    game_instructions(prompt.context).map(|message| InterfaceMainControls {
        node: buttons().build(),
        overlay: GameInstructions::new(message).build(),
        card_anchor_nodes: vec![],
    })
}

fn game_instructions(context: Option<PromptContext>) -> Option<String> {
    match context {
        Some(PromptContext::PlayFromDiscard(card_type)) => Some(format!(
            "Play {} {} from your discard pile.",
            card_type.article(),
            card_type.to_string().to_lowercase()
        )),
        Some(PromptContext::PlayACard) => Some("Play a card.".to_string()),
        Some(PromptContext::PlayNamedCard(name)) => {
            Some(format!("Play {}?", name.displayed_name()))
        }
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
