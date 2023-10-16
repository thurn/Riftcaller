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

use std::cmp;

use adapters::response_builder::ResponseBuilder;
use core_ui::prelude::*;
use game_data::game_actions::{ButtonPrompt, PromptChoice, PromptContext};
use game_data::game_state::GameState;
use game_data::primitives::{CardSubtype, CardType, Milliseconds, Side};
use game_data::tutorial_data::{SpeechBubble, TutorialDisplay};
use prompts::effect_prompts;
use prompts::game_instructions::GameInstructions;
use prompts::prompt_container::PromptContainer;
use protos::spelldawn::{InterfaceMainControls, TutorialEffect};

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
        overlay: prompt_context(game, prompt.context),
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

fn prompt_context(game: &GameState, context: Option<PromptContext>) -> Option<Node> {
    match context {
        Some(PromptContext::CardLimit(card_type, subtype)) => match card_type {
            CardType::Minion => GameInstructions::new(
                "Minion limit exceeded. You must sacrifice a minion in this room.".to_string(),
            ),
            CardType::Artifact if subtype == Some(CardSubtype::Weapon) => GameInstructions::new(
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
        Some(PromptContext::SacrificeToPreventDamage(card_id, amount)) => {
            GameInstructions::new(format!(
                "Sacrifice {} to prevent {} damage?",
                game.card(card_id).variant.name.displayed_name(),
                // Show the total incoming damage if it is lower than the amount to prevent
                cmp::min(
                    amount,
                    game.state_machines.deal_damage.as_ref().map(|d| d.amount).unwrap_or_default()
                )
            ))
            .build()
        }
        _ => None,
    }
}
