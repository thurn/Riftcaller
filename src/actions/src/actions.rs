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
use constants::game_constants;
use game_data::card_state::CardPosition;
use game_data::delegate_data::DrawCardActionEvent;
use game_data::game_actions::{
    BrowserPromptAction, BrowserPromptTarget, BrowserPromptValidation, ButtonPrompt,
    CardBrowserPrompt, CardTarget, GameAction, GamePrompt, GameStateAction, PromptAction,
    PromptChoice, PromptContext,
};
use game_data::game_effect::GameEffect;
use game_data::game_history::HistoryEvent;
use game_data::game_state::{GamePhase, GameState, MulliganDecision, TurnState};
use game_data::game_updates::{GameAnimation, InitiatedBy};
use game_data::primitives::{AbilityId, CardId, RoomId, Side};
use rules::mana::ManaPurpose;
use rules::{
    activate_ability, dispatch, flags, game_effect_actions, mana, mutations, play_card, queries,
};
use tracing::{debug, instrument};
use with_error::{fail, verify, WithError};

/// Top level dispatch function responsible for mutating [GameState] in response
/// to all [GameAction]s
pub fn handle_game_action(
    game: &mut GameState,
    user_side: Side,
    action: &GameAction,
) -> Result<()> {
    if action != &GameAction::Undo && game.undo_tracker.is_some() {
        let clone = game.clone();
        if let Some(undo_tracker) = &mut game.undo_tracker {
            undo_tracker.undo = Some(Box::new(clone));
            undo_tracker.side = Some(user_side);
            undo_tracker.random = false;
            undo_tracker.revealed = false;
        }
    }

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
        GameAction::RemoveCurse => remove_curse_action(game, user_side),
        GameAction::DispelEvocation => dispel_evocation_action(game, user_side),
        GameAction::InitiateRaid(room_id) => {
            raid_state::handle_initiate_action(game, user_side, *room_id)
        }
        GameAction::LevelUpRoom(room_id) => level_up_room_action(game, user_side, *room_id),
        GameAction::SpendActionPoint => spend_action_point_action(game, user_side),
        GameAction::MoveCard(card_id) => move_card_action(game, user_side, *card_id),
        GameAction::RaidAction(action) => raid_state::run(game, Some(*action)),
        GameAction::PromptAction(action) => handle_prompt_action(game, user_side, *action),
        GameAction::Undo => handle_undo_action(game, user_side),
    }?;

    // Clear & store the 'current event' in game history
    game.history.write_events();
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
        game.add_history_event(HistoryEvent::DrawCardAction(*card_id));
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

    play_card::initiate(game, card_id, target)
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

    activate_ability::initiate(game, ability_id, target)
}

/// The basic game action to unveil a project card in play.
#[instrument(skip(game))]
fn unveil_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    verify!(
        flags::can_take_unveil_card_action(game, user_side, card_id),
        "Cannot unveil card {:?}",
        card_id
    );

    game.add_history_event(HistoryEvent::UnveilCard(card_id));
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
    game.add_history_event(HistoryEvent::GainManaAction);
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
    game.add_history_event(HistoryEvent::CardProgress(room_id, 1, InitiatedBy::GameAction));
    mutations::spend_action_points(game, user_side, 1)?;
    mana::spend(game, user_side, ManaPurpose::LevelUpRoom(room_id), 1)?;
    game.add_animation(|| GameAnimation::LevelUpRoom(room_id, InitiatedBy::GameAction));
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
fn remove_curse_action(game: &mut GameState, user_side: Side) -> Result<()> {
    verify!(
        flags::can_take_remove_curse_action(game, user_side),
        "Cannot remove curse for {:?}",
        user_side
    );

    debug!(?user_side, "Applying remove curse action");
    game.add_history_event(HistoryEvent::RemoveCurseAction);
    mutations::spend_action_points(game, user_side, 1)?;
    mana::spend(game, user_side, ManaPurpose::RemoveCurse, game_constants::COST_TO_REMOVE_CURSE)?;
    mutations::remove_curses(game, 1)?;
    Ok(())
}

#[instrument(skip(game))]
fn dispel_evocation_action(game: &mut GameState, user_side: Side) -> Result<()> {
    verify!(
        flags::can_take_dispel_evocation_action(game, user_side),
        "Cannot dispel evocation for {:?}",
        user_side
    );

    debug!(?user_side, "Initiating dispel evocation action");
    game.add_history_event(HistoryEvent::DispelEvocationAction);
    mutations::spend_action_points(game, user_side, 1)?;
    mana::spend(
        game,
        user_side,
        ManaPurpose::RemoveCurse,
        game_constants::COST_TO_DISPEL_EVOCATION,
    )?;

    let prompt = GamePrompt::ButtonPrompt(ButtonPrompt {
        context: None,
        choices: game
            .evocations()
            .map(|card| PromptChoice {
                effects: vec![GameEffect::DestroyCard(card.id)],
                anchor_card: Some(card.id),
                custom_label: None,
            })
            .collect(),
    });
    game.player_mut(user_side).prompt_queue.push(prompt);

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
        (GamePrompt::PlayCardBrowser(_), PromptAction::SkipPlayingCard) => {
            game.player_mut(user_side).prompt_queue.remove(0);
        }
        _ => fail!("Mismatch between active prompt {prompt:?} and action {action:?}"),
    }

    // Try to resume state machines, in case this prompt caused them to pause.
    raid_state::run(game, None)?;
    play_card::run(game)?;
    activate_ability::run(game)
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

fn handle_undo_action(game: &mut GameState, user_side: Side) -> Result<()> {
    verify!(flags::can_take_undo_action(game, user_side), "Cannot currently take undo action");
    let new_state = game
        .undo_tracker
        .as_mut()
        .with_error(|| "Expected undo_tracker")?
        .undo
        .take()
        .with_error(|| "Expected undo state")?;
    *game = *new_state;
    dispatch::populate_delegate_cache(game);
    Ok(())
}
