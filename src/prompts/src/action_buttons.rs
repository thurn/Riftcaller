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
use game_data::game::{GameState, MulliganDecision};
use game_data::game_actions::{
    AccessPhaseAction, CardPromptAction, EncounterAction, PromptAction, SummonAction,
    UnveilProjectAction,
};
use game_data::primitives::Side;
use rules::queries;

use crate::response_button::ResponseButton;

pub fn for_prompt(game: &GameState, side: Side, action: PromptAction) -> ResponseButton {
    match action {
        PromptAction::MulliganDecision(data) => mulligan_button(data),
        PromptAction::SummonAction(data) => summon_button(game, data),
        PromptAction::EncounterAction(data) => encounter_action_button(game, side, data),
        PromptAction::AccessPhaseAction(data) => access_button(data),
        PromptAction::UnveilProjectAction(data) => unveil_button(game, data),
        PromptAction::CardAction(data) => card_response_button(side, data),
    }
    .action(action)
}

fn mulligan_button(mulligan: MulliganDecision) -> ResponseButton {
    match mulligan {
        MulliganDecision::Keep => ResponseButton::new("Keep"),
        MulliganDecision::Mulligan => ResponseButton::new("Mulligan").primary(false),
    }
}

fn summon_button(game: &GameState, summon_action: SummonAction) -> ResponseButton {
    match summon_action {
        SummonAction::SummonMinion(minion_id) => {
            let label = rules::card_definition(game, minion_id).name.displayed_name();
            if let Some(cost) = queries::mana_cost(game, minion_id) {
                if cost > 0 {
                    return ResponseButton::new(format!(
                        "Summon {}\n{}{}",
                        label,
                        cost,
                        icons::MANA
                    ))
                    .two_lines(true);
                }
            }
            ResponseButton::new(format!("Summon {label}"))
        }
        SummonAction::DoNotSummmon => ResponseButton::new("Pass").primary(false),
    }
}

fn encounter_action_button(
    game: &GameState,
    side: Side,
    encounter_action: EncounterAction,
) -> ResponseButton {
    match encounter_action {
        EncounterAction::UseWeaponAbility(source_id, target_id) => {
            let label = rules::card_definition(game, source_id).name.displayed_name();
            if let Some(cost) = queries::cost_to_defeat_target(game, source_id, target_id) {
                if cost > 0 {
                    return ResponseButton::new(format!("{}\n{}{}", label, cost, icons::MANA))
                        .two_lines(true);
                }
            }
            ResponseButton::new(label)
        }
        EncounterAction::NoWeapon => ResponseButton::new("Continue").primary(false),
        EncounterAction::CardAction(action) => card_response_button(side, action),
    }
}

fn access_button(access: AccessPhaseAction) -> ResponseButton {
    match access {
        AccessPhaseAction::ScoreCard(card_id) => ResponseButton::new("Score!").anchor_to(card_id),
        AccessPhaseAction::DestroyCard(card_id, mana) => {
            ResponseButton::new(format!("Destroy\n{}{}", mana, icons::MANA))
                .two_lines(true)
                .anchor_to(card_id)
        }
        AccessPhaseAction::EndRaid => {
            ResponseButton::new("End Raid").primary(false).shift_down(true)
        }
    }
}

fn unveil_button(game: &GameState, unveil_action: UnveilProjectAction) -> ResponseButton {
    match unveil_action {
        UnveilProjectAction::Unveil(project_id) => {
            let label = rules::card_definition(game, project_id).name.displayed_name();
            if let Some(cost) = queries::mana_cost(game, project_id) {
                if cost > 0 {
                    return ResponseButton::new(format!(
                        "Unveil {}\n{}{}",
                        label,
                        cost,
                        icons::MANA
                    ))
                    .two_lines(true);
                }
            }
            ResponseButton::new(format!("Unveil {label}"))
        }
        UnveilProjectAction::DoNotUnveil => ResponseButton::new("Continue").primary(false),
    }
}

fn card_response_button(user_side: Side, action: CardPromptAction) -> ResponseButton {
    let label = match action {
        CardPromptAction::LoseMana(side, amount) => {
            format!("{} {}{}", lose_text(user_side, side), amount, icons::MANA)
        }
        CardPromptAction::LoseActions(side, amount) => {
            if amount > 1 {
                format!("{} {}{}", lose_text(user_side, side), amount, icons::ACTION)
            } else {
                format!("{} {}", lose_text(user_side, side), icons::ACTION)
            }
        }
        CardPromptAction::EndRaid => "End Raid".to_string(),
        CardPromptAction::TakeDamage(_, amount) => format!("Take {amount}"),
        CardPromptAction::TakeDamageEndRaid(_, amount) => format!("End Raid, Take {amount}"),
    };

    ResponseButton::new(label)
}

fn lose_text(user_side: Side, target_side: Side) -> &'static str {
    if user_side == target_side {
        "Pay"
    } else {
        "Lose"
    }
}
