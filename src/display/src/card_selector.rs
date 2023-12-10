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

use adapters::response_builder::ResponseBuilder;
use core_ui::prelude::*;
use game_data::card_state::CardState;
use game_data::game_actions::GameAction;
use game_data::game_state::GameState;
use game_data::prompt_data::{CardSelectorPrompt, GamePrompt, PromptAction, PromptContext};
use prompt_ui::game_instructions::GameInstructions;
use prompt_ui::prompt_container::PromptContainer;
use prompt_ui::response_button::ResponseButton;
use protos::riftcaller::{InterfaceMainControls, ObjectPosition};
use rules::flags;

use crate::positions;

pub fn controls(prompt: &CardSelectorPrompt) -> Option<InterfaceMainControls> {
    Some(InterfaceMainControls {
        node: buttons(prompt).build(),
        overlay: instructions(prompt.context).and_then(|text| {
            GameInstructions::new(text).metatext_optional(metatext(prompt.context)).build()
        }),
        card_anchor_nodes: vec![],
    })
}

fn instructions(context: Option<PromptContext>) -> Option<String> {
    match context {
        Some(PromptContext::DiscardToHandSize(amount)) => {
            Some(format!("You must discard until you have {amount} cards in hand."))
        }
        Some(PromptContext::MoveToTopOfVault) => {
            Some("Put a card from the crypt on top of the vault?".to_string())
        }
        _ => None,
    }
}

fn metatext(context: Option<PromptContext>) -> Option<String> {
    match context {
        Some(PromptContext::DiscardToHandSize(..)) => {
            Some("<i>(Drag cards down from your hand to your deck.)</i>".to_string())
        }
        Some(PromptContext::MoveToTopOfVault) => {
            Some("<i>(Drag cards down from the crypt to the sanctum.)</i>".to_string())
        }
        _ => None,
    }
}

pub fn move_target(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Option<ObjectPosition> {
    let Some(GamePrompt::CardSelector(prompt)) = rules::prompts::current(game, builder.user_side)
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
    let show_submit = flags::card_selector_state_is_valid(prompt);
    PromptContainer::new().child(show_submit.then(|| {
        ResponseButton::new("Submit")
            .action(GameAction::PromptAction(PromptAction::CardSelectorSubmit))
    }))
}
