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

use std::cmp;

use adapters::response_builder::ResponseBuilder;
use core_data::game_primitives::{CardSubtype, CardType, Milliseconds, Side};
use core_ui::icons;
use core_ui::prelude::*;
use game_data::game_actions::ButtonPromptContext;
use game_data::game_state::GameState;
use game_data::prompt_data::{ButtonPrompt, PromptChoice};
use game_data::tutorial_data::{SpeechBubble, TutorialDisplay};
use prompts::effect_prompts;
use prompts::game_instructions::GameInstructions;
use prompts::prompt_container::PromptContainer;
use protos::riftcaller::{InterfaceMainControls, TutorialEffect};
use rules::{curses, damage};

use crate::tutorial_display;

pub fn controls(
    game: &GameState,
    user_side: Side,
    prompt: &ButtonPrompt,
) -> Option<InterfaceMainControls> {
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
        overlay: prompt_context(game, prompt.context.as_ref()),
        card_anchor_nodes,
    })
}

/// Shows a speech bubble for a button prompt. This uses the tutorial effect
/// system, and is intended to show the result when the opponent makes choices.
pub fn append_prompt_speech_bubble<'a>(
    builder: &'a ResponseBuilder,
    game: &'a GameState,
) -> impl Iterator<Item = TutorialEffect> + 'a {
    game.animations.last_prompt_response.iter().filter_map(move |(side, choice)| {
        should_show_bubble(builder, *side, choice).then(|| TutorialEffect {
            tutorial_effect_type: Some(tutorial_display::render_effect(
                builder,
                &TutorialDisplay::SpeechBubble(SpeechBubble {
                    text: effect_prompts::label(*side, choice),
                    side: *side,
                    delay: Milliseconds(0),
                    recurring: false,
                }),
            )),
        })
    })
}

/// Whether a speech bubble should be shown for this user choice.
fn should_show_bubble(builder: &ResponseBuilder, side: Side, choice: &PromptChoice) -> bool {
    builder.user_side != side && !choice.is_secondary() && choice.anchor_card.is_none()
}

fn prompt_context(game: &GameState, prompt_context: Option<&ButtonPromptContext>) -> Option<Node> {
    match prompt_context? {
        ButtonPromptContext::Card(_) => None,
        ButtonPromptContext::PriorityWindow => GameInstructions::new("Activate abilities?").build(),
        ButtonPromptContext::CardLimit(card_type, subtype) => match card_type {
            CardType::Minion => GameInstructions::new(
                "Minion limit exceeded. You must sacrifice a minion in this room.".to_string(),
            ),
            CardType::Artifact if *subtype == Some(CardSubtype::Weapon) => GameInstructions::new(
                "Weapon limit exceeded. You must sacrifice a Weapon card in play.".to_string(),
            ),
            CardType::Project | CardType::Scheme => GameInstructions::new(
                "Sacrifice the existing card occupying this room?".to_string(),
            ),
            _ => GameInstructions::new(format!(
                "{} limit exceeded. You must sacrifice {} {} card in play.",
                card_type,
                match card_type {
                    CardType::Ally | CardType::Artifact | CardType::Evocation => "an",
                    _ => "a",
                },
                card_type
            )),
        }
        .build(),
        ButtonPromptContext::SacrificeToPreventDamage(card_id, amount) => {
            GameInstructions::new(format!(
                "Sacrifice {} to prevent {} damage?",
                game.card(*card_id).variant.name.displayed_name(),
                // Show the total incoming damage if it is lower than the amount to prevent
                cmp::min(*amount, damage::incoming_amount(game).unwrap_or_default())
            ))
            .build()
        }
        ButtonPromptContext::SacrificeToPreventCurses(card_id, amount) => {
            let quantity =
                cmp::min(*amount, curses::incoming_curse_count(game).unwrap_or_default());
            GameInstructions::new(format!(
                "Sacrifice {} to prevent {}?",
                game.card(*card_id).variant.name.displayed_name(),
                if quantity == 1 { "a curse".to_string() } else { format!("{} curses", quantity) }
            ))
            .build()
        }
        ButtonPromptContext::CardToGiveToOpponent => {
            GameInstructions::new("Select card to give opponent").build()
        }
        ButtonPromptContext::CardToTakeFromOpponent => {
            GameInstructions::new("Select card to take from opponent").build()
        }
        ButtonPromptContext::PayToPreventRevealing(cost) => {
            GameInstructions::new(format!("Pay {}{} to prevent revealing?", cost, icons::MANA))
                .build()
        }
    }
}
