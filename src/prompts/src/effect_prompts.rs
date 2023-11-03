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

use core_ui::icons;
use game_data::game_actions::{GameAction, PromptAction, PromptChoice, PromptChoiceLabel};
use game_data::game_effect::GameEffect;
use game_data::primitives::Side;

use crate::response_button::ResponseButton;

/// Builds a [ResponseButton] for a given [PromptChoice].
pub fn button(user_side: Side, index: usize, choice: &PromptChoice) -> ResponseButton {
    let mut result = ResponseButton::new(label(user_side, choice));
    if let Some(card_id) = choice.anchor_card {
        result = result.anchor_to(card_id);
    }
    result
        .action(GameAction::PromptAction(PromptAction::ButtonPromptSelect(index)))
        .primary(!choice.is_secondary())
}

/// Helper to build a button label describing a series of [GameEffect]s.
pub fn label(user_side: Side, choice: &PromptChoice) -> String {
    choice.custom_label.map_or_else(
        || {
            choice
                .effects
                .iter()
                .map(|effect| effect_label(user_side, effect))
                .collect::<Vec<_>>()
                .join(", ")
        },
        custom_label,
    )
}

fn effect_label(user_side: Side, effect: &GameEffect) -> String {
    match effect {
        GameEffect::Continue => "Continue".to_string(),
        GameEffect::AbortPlayingCard => "Cancel".to_string(),
        GameEffect::SacrificeCard(_) => "Sacrifice".to_string(),
        GameEffect::DestroyCard(_) => "Destroy".to_string(),
        GameEffect::LoseMana(side, amount) => {
            format!("{} {}{}", lose_text(user_side, *side), amount, icons::MANA)
        }
        GameEffect::LoseActions(side, amount) => {
            if *amount > 1 {
                format!("{} {}{}", lose_text(user_side, *side), amount, icons::ACTION)
            } else {
                format!("{} {}", lose_text(user_side, *side), icons::ACTION)
            }
        }
        GameEffect::InitiateRaid(_, _) => "Initiate Raid".to_string(),
        GameEffect::EndRaid => "End Raid".to_string(),
        GameEffect::TakeDamage(_, amount) => format!("Take {amount}"),
        GameEffect::MoveCard(_, _) => "Move".to_string(),
        GameEffect::PreventDamage(_) => "Prevent".to_string(),
        GameEffect::PreventCurses(_) => "Prevent".to_string(),
    }
}

fn custom_label(label: PromptChoiceLabel) -> String {
    match label {
        PromptChoiceLabel::Sacrifice => "Sacrifice".to_string(),
        PromptChoiceLabel::Return(cost) => {
            format!("{}{}: Return", cost, icons::MANA)
        }
    }
}

fn lose_text(user_side: Side, target_side: Side) -> &'static str {
    if user_side == target_side {
        "Pay"
    } else {
        "Lose"
    }
}
