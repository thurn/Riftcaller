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

//! Contains functions for responding to user-initiated game actions received
//! from the client. The `handle_user_action` function is the primary
//! entry-point into the rules engine.

pub mod legal_actions;

use anyhow::Result;
use game_data::card_definition::AbilityType;
use game_data::card_state::CardPosition;
use game_data::delegates::{AbilityActivated, ActivateAbilityEvent, DrawCardActionEvent};
use game_data::game::{GamePhase, GameState, MulliganDecision, TurnState};
use game_data::game_actions::{
    BrowserPromptAction, BrowserPromptTarget, BrowserPromptValidation, CardBrowserPrompt,
    CardTarget, GameAction, GamePrompt, GameStateAction, PromptAction, PromptContext,
};
use game_data::primitives::{AbilityId, CardId, RoomId, Side};
use game_data::updates::{GameUpdate, InitiatedBy};
use rules::mana::ManaPurpose;
use rules::{dispatch, flags, game_effect_actions, mana, mutations, play_card, queries};
use tracing::{debug, instrument};
use with_error::{fail, verify, WithError};

/// Top level dispatch function responsible for mutating [GameState] in response
/// to all [GameAction]s
pub fn handle_game_action(
    game: &mut GameState,
    user_side: Side,
    action: &GameAction,
) -> Result<()> {
    match action {
        GameAction::GameStateAction(action) => handle_game_state_action(game, user_side, *action),
        GameAction::Resign => handle_resign_action(game, user_side),
        GameAction::GainMana => gain_mana_action(game, user_side),
        GameAction::DrawCard => draw_card_action(game, user_side),
        GameAction::PlayCard(card_id, target) => {
            play_card_action(game, user_side, *card_id, *target)
        }
        GameAction::ActivateAbility(ability_id, target) => {
            activate_ability_action(game, user_side, *ability_id, *target)
        }
        GameAction::UnveilCard(card_id) => unveil_action(game, user_side, *card_id),
        GameAction::InitiateRaid(room_id) => {
            raids::handle_initiate_action(game, user_side, *room_id)
        }
        GameAction::LevelUpRoom(room_id) => level_up_room_action(game, user_side, *room_id),
        GameAction::SpendActionPoint => spend_action_point_action(game, user_side),
        GameAction::MoveCard(card_id) => move_card_action(game, user_side, *card_id),
        GameAction::RaidAction(action) => raids::run(game, Some(*action)),
        GameAction::PromptAction(action) => handle_prompt_action(game, user_side, *action),
    }?;

    Ok(())
}

fn handle_resign_action(game: &mut GameState, side: Side) -> Result<()> {
    debug!(?side, "Applying resign action");
    if !matches!(game.info.phase, GamePhase::GameOver { .. }) {
        mutations::game_over(game, side.opponent())?;
    }
    Ok(())
}

/// The basic game action to draw a card during your turn by spending one
/// action.
#[instrument(skip(game))]
fn draw_card_action(game: &mut GameState, user_side: Side) -> Result<()> {
    verify!(
        flags::can_take_draw_card_action(game, user_side),
        "Cannot draw card for {:?}",
        user_side
    );

    debug!(?user_side, "Applying draw card action");
    mutations::spend_action_points(game, user_side, 1)?;
    let cards = mutations::draw_cards(game, user_side, 1)?;
    if let Some(card_id) = cards.get(0) {
        dispatch::invoke_event(game, DrawCardActionEvent(*card_id))?;
    }

    Ok(())
}

/// The basic game action to play a card during your turn. Spends the resource
/// cost for a card, resolves its effects, and then moves it to the appropriate
/// new [CardPosition]. Spell, Weapon, and Artifact cards are immediately
/// revealed when played.
#[instrument(skip(game))]
fn play_card_action(
    game: &mut GameState,
    user_side: Side,
    card_id: CardId,
    target: CardTarget,
) -> Result<()> {
    verify!(
        flags::can_take_play_card_action(game, user_side, card_id, target),
        "Cannot play card {:?}",
        card_id
    );

    play_card::run(game, card_id, target)
}

/// The basic game action to activate an ability of a card in play.
#[instrument(skip(game))]
fn activate_ability_action(
    game: &mut GameState,
    user_side: Side,
    ability_id: AbilityId,
    target: CardTarget,
) -> Result<()> {
    verify!(
        flags::can_take_activate_ability_action(game, user_side, ability_id, target),
        "Cannot activate ability {:?}",
        ability_id
    );

    game.ability_state.entry(ability_id).or_default().currently_resolving = true;
    let card = game.card(ability_id.card_id);

    debug!(?card.name, ?user_side, ?ability_id, "Applying activate ability action");

    let cost = match &rules::get(card.name).ability(ability_id.index).ability_type {
        AbilityType::Activated(cost, _) => cost,
        _ => fail!("Ability is not an activated ability"),
    };

    mutations::spend_action_points(game, user_side, cost.actions)?;
    if let Some(mana) = queries::ability_mana_cost(game, ability_id) {
        mana::spend(game, user_side, ManaPurpose::ActivateAbility(ability_id), mana)?;
    }

    if let Some(custom_cost) = &cost.custom_cost {
        (custom_cost.pay)(game, ability_id)?;
    }

    game.record_update(|| GameUpdate::AbilityActivated(user_side, ability_id));
    dispatch::invoke_event(game, ActivateAbilityEvent(AbilityActivated { ability_id, target }))?;

    game.ability_state.entry(ability_id).or_default().currently_resolving = false;
    Ok(())
}

/// The basic game action to unveil a project card in play.
#[instrument(skip(game))]
fn unveil_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    verify!(
        flags::can_take_unveil_card_action(game, user_side, card_id),
        "Cannot unveil card {:?}",
        card_id
    );

    mutations::unveil_card(game, card_id)?;
    Ok(())
}

/// The basic game action to gain 1 mana during your turn by spending one
/// action.
#[instrument(skip(game))]
fn gain_mana_action(game: &mut GameState, user_side: Side) -> Result<()> {
    verify!(
        flags::can_take_gain_mana_action(game, user_side),
        "Cannot gain mana for {:?}",
        user_side
    );

    debug!(?user_side, "Applying gain mana action");
    mutations::spend_action_points(game, user_side, 1)?;
    mana::gain(game, user_side, 1);
    Ok(())
}

fn level_up_room_action(game: &mut GameState, user_side: Side, room_id: RoomId) -> Result<()> {
    verify!(
        flags::can_take_level_up_room_action(game, user_side, room_id),
        "Cannot level up room for {:?}",
        user_side
    );
    debug!(?user_side, "Applying level up room action");
    mutations::spend_action_points(game, user_side, 1)?;
    mana::spend(game, user_side, ManaPurpose::LevelUpRoom(room_id), 1)?;
    game.record_update(|| GameUpdate::LevelUpRoom(room_id, InitiatedBy::GameAction));
    mutations::level_up_room(game, room_id)?;
    Ok(())
}

fn spend_action_point_action(game: &mut GameState, user_side: Side) -> Result<()> {
    verify!(
        flags::in_main_phase_with_action_point(game, user_side),
        "Cannot spend action point for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1)?;
    Ok(())
}

#[instrument(skip(game))]
fn move_card_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    let Some(GamePrompt::CardBrowserPrompt(prompt)) =
        game.player_mut(user_side).prompt_queue.get_mut(0)
    else {
        fail!("Expected active CardBrowserPrompt");
    };

    if let Some(position) = prompt.chosen_subjects.iter().position(|id| *id == card_id) {
        prompt.chosen_subjects.remove(position);
        prompt.unchosen_subjects.push(card_id);
    } else if let Some(position) = prompt.unchosen_subjects.iter().position(|id| *id == card_id) {
        prompt.unchosen_subjects.remove(position);
        prompt.chosen_subjects.push(card_id);
    } else {
        fail!("Expected card to be a subject of the active browser");
    }
    Ok(())
}

/// Handles a [GameStateAction] for the `user_side` player and then removes it
/// from the queue.
fn handle_game_state_action(
    game: &mut GameState,
    user_side: Side,
    action: GameStateAction,
) -> Result<()> {
    verify!(flags::can_take_game_state_actions(game, user_side), "Cannot currently act");
    match action {
        GameStateAction::MulliganDecision(mulligan) => {
            handle_mulligan_decision(game, user_side, mulligan)
        }
        GameStateAction::StartTurnAction => handle_start_turn_action(game, user_side),
        GameStateAction::EndTurnAction => handle_end_turn_action(game, user_side),
    }
}

/// Handles a choice to keep or mulligan an opening hand
fn handle_mulligan_decision(
    game: &mut GameState,
    user_side: Side,
    decision: MulliganDecision,
) -> Result<()> {
    verify!(
        flags::can_make_mulligan_decision(game, user_side),
        "Cannot make mulligan decision for {:?}",
        user_side
    );

    debug!(?user_side, ?decision, "Applying mulligan action");
    let mulligans = match &mut game.info.phase {
        GamePhase::ResolveMulligans(mulligans) => mulligans,
        _ => fail!("Incorrect game phase"),
    };

    match user_side {
        Side::Overlord => mulligans.overlord = Some(decision),
        Side::Champion => mulligans.champion = Some(decision),
    }

    let hand = game.card_list_for_position(user_side, CardPosition::Hand(user_side));
    match decision {
        MulliganDecision::Keep => {}
        MulliganDecision::Mulligan => {
            mutations::shuffle_into_deck(game, user_side, &hand)?;
            mutations::draw_cards(game, user_side, 5)?;
        }
    }

    mutations::check_start_game(game)?;

    Ok(())
}

fn handle_start_turn_action(game: &mut GameState, user_side: Side) -> Result<()> {
    verify!(flags::can_take_start_turn_action(game, user_side), "Cannot start turn");
    start_next_turn(game)
}

fn handle_end_turn_action(game: &mut GameState, user_side: Side) -> Result<()> {
    verify!(flags::can_take_end_turn_action(game, user_side), "Cannot end turn");

    let turn = game.info.turn;
    let side = turn.side;

    let max_hand_size = queries::maximum_hand_size(game, side) as usize;
    let hand = game.card_list_for_position(side, CardPosition::Hand(side));
    game.info.turn_state = TurnState::Ended;

    if hand.len() > max_hand_size {
        // Must discard to hand size
        let discard = hand.len() - max_hand_size;
        game.player_mut(user_side).prompt_queue.push(GamePrompt::CardBrowserPrompt(
            CardBrowserPrompt {
                context: Some(PromptContext::DiscardToHandSize(max_hand_size)),
                unchosen_subjects: hand,
                chosen_subjects: vec![],
                target: BrowserPromptTarget::DiscardPile,
                validation: BrowserPromptValidation::ExactlyCount(discard),
                action: BrowserPromptAction::DiscardCards,
            },
        ));
        Ok(())
    } else {
        check_start_next_turn(game)
    }
}

fn check_start_next_turn(game: &mut GameState) -> Result<()> {
    let side = game.info.turn.side;
    let ended = game.player(side).actions == 0;

    // Next turn immediately starts unless the current player is the Champion
    // and the Overlord can unveil a Duskbound project.
    if ended && (side == Side::Overlord || !flags::overlord_has_instant_speed_actions(game)) {
        start_next_turn(game)
    } else {
        Ok(())
    }
}

fn start_next_turn(game: &mut GameState) -> Result<()> {
    let current_side = game.info.turn.side;
    mutations::start_turn(
        game,
        current_side.opponent(),
        match current_side {
            Side::Overlord => game.info.turn.turn_number,
            Side::Champion => game.info.turn.turn_number + 1,
        },
    )
}

fn handle_prompt_action(game: &mut GameState, user_side: Side, action: PromptAction) -> Result<()> {
    let Some(prompt) = game.player(user_side).prompt_queue.get(0) else {
        fail!("Expected active GamePrompt");
    };

    match (prompt, action) {
        (GamePrompt::ButtonPrompt(buttons), PromptAction::ButtonPromptSelect(index)) => {
            let choice =
                buttons.choices.get(index).with_error(|| format!("Index out of bounds {index}"))?;

            let effects = choice.effects.clone();
            for effect in effects {
                game_effect_actions::handle(game, effect)?;
            }

            game.player_mut(user_side).prompt_queue.remove(0);
        }
        (GamePrompt::CardBrowserPrompt(browser), PromptAction::CardBrowserPromptSubmit) => {
            handle_card_browser_submit_action(
                game,
                user_side,
                browser.chosen_subjects.clone(),
                browser.action,
            )?;
        }
        _ => fail!("Mismatch between active prompt {prompt:?} and action {action:?}"),
    }

    // Try to resume the raid state machine, in case this prompt caused it to pause.
    raids::run(game, None)
}

fn handle_card_browser_submit_action(
    game: &mut GameState,
    user_side: Side,
    subjects: Vec<CardId>,
    action: BrowserPromptAction,
) -> Result<()> {
    match action {
        BrowserPromptAction::DiscardCards => {
            for card_id in subjects {
                mutations::move_card(game, card_id, CardPosition::DiscardPile(card_id.side))?;
            }
        }
    }

    game.player_mut(user_side).prompt_queue.remove(0);
    check_start_next_turn(game)
}
