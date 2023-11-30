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
use game_data::game_actions::{RoomSelectorPrompt, RoomSelectorPromptContext};
use prompts::game_instructions::GameInstructions;
use protos::spelldawn::InterfaceMainControls;

pub fn controls(prompt: &RoomSelectorPrompt) -> Option<InterfaceMainControls> {
    Some(InterfaceMainControls {
        node: None,
        overlay: instructions(prompt.context).and_then(|text| {
            GameInstructions::new(text).metatext("<i>(Drag card to target room)</i>").build()
        }),
        card_anchor_nodes: vec![],
    })
}

fn instructions(context: Option<RoomSelectorPromptContext>) -> Option<String> {
    match context {
        Some(RoomSelectorPromptContext::Access) => Some("Select a room to access".to_string()),
        _ => None,
    }
}
