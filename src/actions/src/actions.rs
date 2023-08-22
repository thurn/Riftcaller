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
use game_data::delegates::{
    AbilityActivated, ActivateAbilityEvent, CardPlayed, CastCardEvent, DrawCardActionEvent,
};
use game_data::game::{GamePhase, GameState, HistoryEntry, HistoryEvent, MulliganDecision};
use game_data::game_actions::{CardTarget, GameAction, PromptAction};
use game_data::primitives::{AbilityId, CardId, RoomId, Side};
use game_data::updates::{GameUpdate, InitiatedBy};
use raids::RaidDataExt;
use rules::mana::ManaPurpose;
use rules::{card_prompt, dispatch, flags, mana, mutations, queries};
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
        GameAction::PromptAction(prompt_action) => {
            handle_prompt_action(game, user_side, *prompt_action)
        }
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
    }?;

    Ok(())
}

/// Returns true if the indicated player currently has a legal game action
/// available to them.
pub fn has_priority(game: &GameState, side: Side) -> bool {
    match &game.info.phase {
        GamePhase::ResolveMulligans(_) => return flags::can_make_mulligan_decision(game, side),
        GamePhase::GameOver { .. } => return false,
        _ => {}
    };

    match &game.info.raid {
        Some(raid) => side == raid.phase().active_side(),
        None => side == game.info.turn.side,
    }
}

fn handle_resign_action(game: &mut GameState, side: Side) -> Result<()> {
    debug!(?side, "Applying resign action");
    if !matches!(game.info.phase, GamePhase::GameOver { .. }) {
        mutations::game_over(game, side.opponent())?;
    }
    Ok(())
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
    let mut mulligans = match &mut game.info.phase {
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

    mutations::check_end_turn(game)?;
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

    let card = game.card(card_id);
    debug!(?card.name, ?user_side, ?card_id, ?target, "Applying play card action");

    let definition = rules::get(card.name);
    mutations::move_card(game, card_id, CardPosition::Played(user_side, target))?;

    let actions = queries::action_cost(game, card_id);
    mutations::spend_action_points(game, user_side, actions)?;

    if flags::enters_play_face_up(game, card_id) {
        let amount = queries::mana_cost(game, card_id).with_error(|| "Card has no mana cost")?;
        mana::spend(game, user_side, ManaPurpose::PayForCard(card_id), amount)?;
        if let Some(custom_cost) = &definition.cost.custom_cost {
            (custom_cost.pay)(game, card_id)?;
        }
        game.card_mut(card_id).turn_face_up();
        game.record_update(|| GameUpdate::PlayCardFaceUp(user_side, card_id));
    }

    dispatch::invoke_event(game, CastCardEvent(CardPlayed { card_id, target }))?;
    mutations::move_card(
        game,
        card_id,
        queries::played_position(game, user_side, card_id, target)?,
    )?;

    game.history
        .push(HistoryEntry { turn: game.info.turn, event: HistoryEvent::PlayedCard(card_id) });
    mutations::check_end_turn(game)?;
    Ok(())
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
    mutations::check_end_turn(game)?;
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
    mutations::check_end_turn(game)?;
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
    mutations::check_end_turn(game)?;
    Ok(())
}

fn spend_action_point_action(game: &mut GameState, user_side: Side) -> Result<()> {
    verify!(flags::in_main_phase(game, user_side), "Cannot spend action point for {:?}", user_side);
    mutations::spend_action_points(game, user_side, 1)?;
    mutations::check_end_turn(game)?;
    Ok(())
}

/// Handles a [PromptAction] for the `user_side` player and then removes it from
/// the queue.
fn handle_prompt_action(game: &mut GameState, user_side: Side, action: PromptAction) -> Result<()> {
    if let Some(prompt) = &game.player(user_side).card_prompt_queue.get(0) {
        verify!(
            prompt.responses.iter().any(|p| *p == action),
            "Unexpected action {:?} received",
            action
        );

        game.player_mut(user_side).card_prompt_queue.remove(0);
    } else if matches!(action, PromptAction::CardAction(_)) {
        fail!("Received action with no matching prompt {:?}", action);
    }

    match action {
        PromptAction::MulliganDecision(mulligan) => {
            handle_mulligan_decision(game, user_side, mulligan)
        }
        PromptAction::SummonAction(_)
        | PromptAction::EncounterAction(_)
        | PromptAction::AccessPhaseAction(_) => raids::handle_action(game, user_side, action),
        PromptAction::CardAction(card_action) => card_prompt::handle(game, user_side, card_action),
    }
}
