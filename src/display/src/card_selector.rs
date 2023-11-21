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

use adapters::response_builder::ResponseBuilder;
use core_ui::prelude::*;
use game_data::card_state::CardState;
use game_data::game_actions::{
    BrowserPromptValidation, CardSelectorPrompt, GameAction, GamePrompt, PromptAction,
    PromptContext,
};
use game_data::game_state::GameState;
use prompts::game_instructions::GameInstructions;
use prompts::prompt_container::PromptContainer;
use prompts::response_button::ResponseButton;
use protos::spelldawn::{InterfaceMainControls, ObjectPosition};

use crate::positions;

pub fn controls(prompt: &CardSelectorPrompt) -> Option<InterfaceMainControls> {
    match prompt.context {
        Some(PromptContext::DiscardToHandSize(amount)) => Some(InterfaceMainControls {
            node: buttons(prompt).build(),
            overlay: GameInstructions::new(format!(
                "You must discard until you have {amount} cards in hand."
            ))
            .metatext("<i>(Drag cards down from your hand to your discard pile.)</i>")
            .build(),
            card_anchor_nodes: vec![],
        }),
        _ => None,
    }
}

pub fn move_target(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Option<ObjectPosition> {
    let Some(GamePrompt::CardSelector(prompt)) =
        game.player(builder.user_side).prompt_stack.current()
    else {
        return None;
    };

    if prompt.unchosen_subjects.contains(&card.id) {
        Some(positions::for_card(card, positions::card_browser_target_position()))
    } else if prompt.chosen_subjects.contains(&card.id) {
        Some(positions::for_card(card, positions::revealed_cards(true)))
    } else {
        None
    }
}

fn buttons(prompt: &CardSelectorPrompt) -> PromptContainer {
    let show_submit = is_valid(prompt);
    PromptContainer::new().child(show_submit.then(|| {
        ResponseButton::new("Submit")
            .action(GameAction::PromptAction(PromptAction::CardSelectorSubmit))
    }))
}

fn is_valid(prompt: &CardSelectorPrompt) -> bool {
    match prompt.validation {
        BrowserPromptValidation::ExactlyCount(count) => prompt.chosen_subjects.len() == count,
    }
}
