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
use game_data::game::GameState;
use game_data::game_actions::ButtonPrompt;
use game_data::primitives::Side;
use protos::spelldawn::InterfaceMainControls;

use crate::action_buttons;
use crate::prompt_container::PromptContainer;

/// Builds UI elements to display a [ButtonPrompt] for the `side` player.
pub fn action_prompt(
    game: &GameState,
    side: Side,
    prompt: &ButtonPrompt,
) -> Option<InterfaceMainControls> {
    let mut main_controls: Vec<Box<dyn ComponentObject>> = vec![];
    let mut card_anchor_nodes = vec![];

    for response in &prompt.responses {
        let button = action_buttons::for_prompt(game, side, *response);
        if button.has_anchor() {
            card_anchor_nodes.push(button.render_to_card_anchor_node());
        } else {
            main_controls.push(Box::new(button));
        }
    }

    Some(InterfaceMainControls {
        node: PromptContainer::new().children(main_controls).build(),
        overlay: None,
        card_anchor_nodes,
    })
}
