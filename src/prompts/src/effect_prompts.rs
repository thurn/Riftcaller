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

use core_data::game_primitives::Side;
use core_ui::icons;
use game_data::game_actions::{GameAction, PromptAction, PromptChoice, PromptChoiceLabel};
use game_data::game_effect::GameEffect;

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
        GameEffect::PlayChoiceEffect { .. } => String::new(),
        GameEffect::SacrificeCard(..) => "Sacrifice".to_string(),
        GameEffect::DestroyCard(..) => "Destroy".to_string(),
        GameEffect::ManaCost(side, amount, ..) => {
            format!("{} {}{}", lose_text(user_side, *side), amount, icons::MANA)
        }
        GameEffect::ActionCost(side, amount) => {
            if *amount > 1 {
                format!("{} {}{}", lose_text(user_side, *side), amount, icons::ACTION)
            } else {
                format!("{} {}", lose_text(user_side, *side), icons::ACTION)
            }
        }
        GameEffect::InitiateRaid(..) => "Initiate Raid".to_string(),
        GameEffect::EndRaid(..) => "End Raid".to_string(),
        GameEffect::EndCustomAccess(..) => "End Access".to_string(),
        GameEffect::TakeDamageCost(_, amount) => format!("Take {amount}"),
        GameEffect::MoveCard(..) => "Move".to_string(),
        GameEffect::PreventDamage(..) => "Prevent".to_string(),
        GameEffect::PreventCurses(..) => "Prevent".to_string(),
        GameEffect::SelectCardForPrompt(..) => "Select".to_string(),
        GameEffect::SwapWithSelected(..) => "Swap".to_string(),
        GameEffect::SetOnPlayState { .. } => "Choose".to_string(),
        GameEffect::RecordCardChoice(..) => "Choose".to_string(),
        GameEffect::AppendCustomCardState(..) => "Choose".to_string(),
        GameEffect::EvadeCurrentEncounter => "Evade".to_string(),
        GameEffect::PlayCardForNoMana(..) => "Play".to_string(),
        GameEffect::PreventRaidCardAccess => "Do not Access".to_string(),
    }
}

fn custom_label(label: PromptChoiceLabel) -> String {
    match label {
        PromptChoiceLabel::Play => "Play".to_string(),
        PromptChoiceLabel::Sacrifice => "Sacrifice".to_string(),
        PromptChoiceLabel::Return(cost) => {
            format!("{}{}: Return", cost, icons::MANA)
        }
        PromptChoiceLabel::Occupant => "Occupant".to_string(),
        PromptChoiceLabel::Defender => "Defender".to_string(),
        PromptChoiceLabel::PayActionAccessAnother => format!("Access Another? ({})", icons::ACTION),
    }
}

fn lose_text(user_side: Side, target_side: Side) -> &'static str {
    if user_side == target_side {
        "Pay"
    } else {
        "Lose"
    }
}
