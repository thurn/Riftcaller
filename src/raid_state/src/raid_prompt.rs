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

use core_data::game_primitives::{CardId, RoomId, Side};
use core_ui::design::BackgroundColor;
use core_ui::icons;
use core_ui::prelude::*;
use core_ui::style::Corner;
use core_ui::text::Text;
use game_data::card_configuration::{Cost, CustomBoostCost, CustomWeaponCost};
use game_data::game_actions::{GameAction, RaidAction, RazeCardActionType};
use game_data::game_state::GameState;
use game_data::raid_data::{
    RaidChoice, RaidData, RaidLabel, RaidPrompt, RaidState, RaidStep, WeaponInteraction,
};
use game_data::tutorial_data::TutorialTrigger;
use prompt_ui::prompt_container::PromptContainer;
use prompt_ui::response_button::ResponseButton;
use protos::riftcaller::{
    AnchorCorner, CardAnchor, CardAnchorNode, FlexAlign, FlexJustify, InterfaceMainControls,
};
use rules::combat::CustomBoostActivation;
use rules::{combat, queries, CardDefinitionExt};

/// Builds an [InterfaceMainControls] response to show to the `side` player in
/// order to make a decision in this raid if a choice is currently available.
pub fn build(game: &GameState, raid: &RaidData, side: Side) -> Option<InterfaceMainControls> {
    if let Some(prompt) = current_prompt(game, side) {
        let mut main_controls: Vec<Box<dyn ComponentObject>> = vec![];
        let mut card_anchor_nodes = vec![];

        for (i, choice) in prompt.choices.iter().enumerate() {
            let button = render_button(game, i, choice);
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
    } else if raid.target == RoomId::Sanctum && side == Side::Covenant && !raid.accessed.is_empty()
    {
        // During sanctum access, display an annotation on accessed cards for the
        // Covenant
        covenant_sanctum_access_info(raid)
    } else {
        None
    }
}

/// Returns a vector of the [GameAction]s that are currently available for the
/// `side` player to take in the current raid state.
pub fn legal_actions(game: &GameState, side: Side) -> Vec<GameAction> {
    current_prompt(game, side).map_or_else(Vec::new, |prompt| {
        prompt
            .choices
            .iter()
            .enumerate()
            .map(|(i, _)| GameAction::RaidAction(RaidAction { index: i }))
            .collect()
    })
}

/// Returns the current raid prompt for the `side` user, if any.
fn current_prompt(game: &GameState, side: Side) -> Option<&RaidPrompt> {
    if let Some(RaidState::Prompt(prompt)) = game.raid.as_ref().map(|r| &r.state) {
        if prompt.status.side() == side {
            return Some(prompt);
        }
    }

    None
}

fn covenant_sanctum_access_info(raid: &RaidData) -> Option<InterfaceMainControls> {
    Some(InterfaceMainControls {
        node: None,
        overlay: None,
        card_anchor_nodes: raid.accessed.iter().map(sanctum_access_note).collect(),
    })
}

fn sanctum_access_note(card_id: &CardId) -> CardAnchorNode {
    CardAnchorNode {
        card_id: Some(adapters::card_identifier(*card_id)),
        node: Row::new("CardAccessInfo")
            .style(
                Style::new()
                    .margin(Edge::Top, 16.px())
                    .margin(Edge::Horizontal, 32.px())
                    .padding(Edge::All, 8.px())
                    .justify_content(FlexJustify::Center)
                    .align_items(FlexAlign::Center)
                    .border_radius(Corner::All, 8.px())
                    .background_color(BackgroundColor::AccessedNote),
            )
            .child(Text::new(format!("{} Access", icons::EYE)))
            .build(),
        anchors: vec![
            CardAnchor {
                node_corner: AnchorCorner::TopLeft as i32,
                card_corner: AnchorCorner::BottomLeft as i32,
            },
            CardAnchor {
                node_corner: AnchorCorner::TopRight as i32,
                card_corner: AnchorCorner::BottomRight as i32,
            },
        ],
    }
}

/// Checks whether the provided [RaidAction] corresponds to the effect for a
/// [TutorialTrigger].
pub fn matches_tutorial_trigger(
    game: &GameState,
    raid_action: RaidAction,
    trigger: &TutorialTrigger,
) -> bool {
    let Some(RaidState::Prompt(prompt)) = &game.raid.as_ref().map(|r| &r.state) else {
        return false;
    };

    let Some(choice) = prompt.choices.get(raid_action.index) else {
        return false;
    };

    match (trigger, &choice.step) {
        (TutorialTrigger::SummonMinion(name), RaidStep::SummonMinion(id)) => {
            game.card(*id).variant.name == *name
        }
        (TutorialTrigger::UseWeapon { weapon, target }, RaidStep::UseWeapon(interaction)) => {
            game.card(interaction.weapon_id).variant.name == *weapon
                && game.card(interaction.defender_id).variant.name == *target
        }
        (TutorialTrigger::UseNoWeapon, RaidStep::FireMinionCombatAbility(_)) => true,
        (TutorialTrigger::ScoreAccessedCard(name), RaidStep::StartScoringCard(card)) => {
            game.card(card.id).variant.name == *name
        }
        (TutorialTrigger::SuccessfullyEndRaid, RaidStep::FinishRaid) => true,
        _ => false,
    }
}

fn render_button(game: &GameState, index: usize, choice: &RaidChoice) -> ResponseButton {
    let button = match choice.label {
        RaidLabel::SummonMinion(minion_id) => summon_button(game, minion_id),
        RaidLabel::DoNotSummonMinion => ResponseButton::new("Pass").primary(false),
        RaidLabel::UseWeapon(interaction) => use_weapon_button(game, interaction),
        RaidLabel::DoNotUseWeapon => ResponseButton::new("Continue").primary(false),
        RaidLabel::ProceedToAccess => ResponseButton::new("Proceed to Access"),
        RaidLabel::ScoreCard(card_id) => score_button(game, card_id),
        RaidLabel::RazeCard(card_id, action) => raze_button(game, card_id, action),
        RaidLabel::EndRaid => ResponseButton::new("End Raid").primary(false).shift_down(true),
        RaidLabel::EndAccess => ResponseButton::new("End Access").primary(false).shift_down(true),
    };

    button.action(GameAction::RaidAction(RaidAction { index }))
}

fn summon_button(game: &GameState, minion_id: CardId) -> ResponseButton {
    let label = game.card(minion_id).definition().name.displayed_name();
    if let Some(cost) = queries::mana_cost(game, minion_id) {
        if cost > 0 {
            return ResponseButton::new(format!("Summon {}\n{}{}", label, cost, icons::MANA))
                .two_lines(true);
        }
    }
    ResponseButton::new(format!("Summon {label}"))
}

fn use_weapon_button(game: &GameState, interaction: WeaponInteraction) -> ResponseButton {
    let label = game.card(interaction.weapon_id).definition().name.displayed_name();
    if let Some(cost_to_defeat) =
        combat::cost_to_defeat_target(game, interaction.weapon_id, interaction.defender_id)
    {
        let prepend = cost_to_defeat.custom_weapon_cost.as_ref().map(custom_weapon_cost_label);

        if let Some(activation) = cost_to_defeat.custom_boost_activation.as_ref() {
            return ResponseButton::new(custom_weapon_activation_label(activation, label))
                .two_lines(true);
        }

        if cost_to_defeat.mana_cost > 0 {
            return ResponseButton::new(format!(
                "{}\n{}{}{}",
                label,
                prepend.unwrap_or(String::new()),
                cost_to_defeat.mana_cost,
                icons::MANA
            ))
            .two_lines(true);
        }
    }
    ResponseButton::new(label)
}

fn custom_weapon_cost_label(cost: &CustomWeaponCost) -> String {
    match cost {
        CustomWeaponCost::ActionPoints(points) => {
            format!("{},", icons::ACTION.repeat(*points as usize))
        }
    }
}

fn custom_weapon_activation_label(activation: &CustomBoostActivation, label: String) -> String {
    match activation.cost {
        CustomBoostCost::PowerCharges(n) => {
            format!("{}\n{}{}", label, n * activation.activation_count, icons::POWER_CHARGE)
        }
    }
}

fn score_button(game: &GameState, card_id: CardId) -> ResponseButton {
    let cost = cost_string(queries::score_accessed_card_cost(game, card_id));
    if cost.is_empty() {
        ResponseButton::new("Score!").anchor_to(card_id)
    } else {
        ResponseButton::new(format!("Score!\n{}", cost)).two_lines(true).anchor_to(card_id)
    }
}

fn cost_string<T>(cost: Cost<T>) -> String {
    let costs = vec![
        cost.mana.map(|m| format!("{}{}", m, icons::MANA)),
        (cost.actions > 0).then(|| format!("{},", icons::ACTION.repeat(cost.actions as usize))),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<String>>();

    costs.join(",")
}

fn raze_button(game: &GameState, card_id: CardId, action: RazeCardActionType) -> ResponseButton {
    let mana = queries::raze_cost(game, card_id);
    let label = match action {
        RazeCardActionType::Destroy => format!("Destroy\n{}{}", mana, icons::MANA),
        RazeCardActionType::Discard => format!("Discard\n{}{}", mana, icons::MANA),
    };
    ResponseButton::new(label).two_lines(true).anchor_to(card_id)
}
