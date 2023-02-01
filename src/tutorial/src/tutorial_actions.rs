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

use anyhow::Result;
use game_data::card_name::CardName;
use game_data::card_state::CardPosition;
use game_data::game::{GameState, MulliganDecision};
use game_data::game_actions::{AccessPhaseAction, EncounterAction, GameAction, PromptAction};
use game_data::primitives::{CardId, RoomLocation, Side};
use game_data::tutorial_data::{
    TutorialDisplay, TutorialGameStateTrigger, TutorialOpponentAction, TutorialStep,
    TutorialTrigger,
};
use rules::mutations;
use tracing::{debug, debug_span, info};
use with_error::{fail, WithError};

/// Handle applying tutorial actions before evaluating the effects a given
/// [GameAction].
///
/// The tutorial is broken up into two distinct steps: 1) a pre-scripted
/// tutorial sequence, applied by [handle_sequence_game_action], and 2) a
/// reactive system which provides contextual help when certain user actions are
/// taken, applied by `handle_triggered_action`.
pub fn handle_game_action(game: &mut GameState, action: Option<&GameAction>) -> Result<()> {
    if action.is_some() {
        game.data.tutorial_state.display.retain(|display| display.recurring());
    }

    if game.data.config.scripted_tutorial
        && game.data.tutorial_state.index < crate::SEQUENCE.steps.len()
    {
        handle_sequence_game_action(game, action)?;
    }

    handle_triggered_action(game, action)
}

/// Handle applying tutorial actions for the pre-scripted tutorial sequence.
/// This is used at the beginning of the tutorial game, when all behavior is
/// pre-determined.
pub fn handle_sequence_game_action(
    game: &mut GameState,
    mut user_action: Option<&GameAction>,
) -> Result<()> {
    let _span = debug_span!("handle_sequence_game_action").entered();
    let mut i = game.data.tutorial_state.index;

    while i < crate::SEQUENCE.steps.len() {
        let action = &crate::SEQUENCE.steps[i];
        let _span = debug_span!("handle_tutorial_action", ?action).entered();
        debug!(?action, "Handling tutorial action");

        match action {
            TutorialStep::KeepOpeningHand(side) => keep_opening_hand(game, *side),
            TutorialStep::SetHand(side, cards) => set_hand(game, *side, cards),
            TutorialStep::SetTopOfDeck(side, cards) => set_top_of_deck(game, *side, cards),
            TutorialStep::OpponentAction(action) => {
                if match_opponent_action(game, user_action, action)? {
                    user_action = None; // Consume action, avoid matching again
                } else {
                    debug!(?action, "Awaiting oppponent action");
                    break;
                }
                Ok(())
            }
            TutorialStep::DefaultOpponentAction(_) => Ok(()),
            TutorialStep::AwaitTriggers(actions) => {
                if await_player_actions(game, user_action, actions)? {
                    user_action = None; // Consume action, avoid matching again

                    // Clear recurring messages
                    game.data.tutorial_state.display.clear();
                } else {
                    debug!(?actions, "Awaiting user action");
                    break;
                }
                Ok(())
            }
            TutorialStep::AwaitGameState(trigger) => {
                if !game_state_matches(game, trigger) {
                    debug!(?trigger, "Awaiting game state");
                    break;
                }
                Ok(())
            }
            TutorialStep::Display(displays) => display(game, displays.clone()),
            TutorialStep::AddGameModifiers(card_names) => add_game_modifiers(game, card_names),
            TutorialStep::RemoveGameModifiers(card_names) => {
                remove_game_modifiers(game, card_names)
            }
        }?;

        i += 1;
    }

    game.data.tutorial_state.index = i;
    if i < crate::SEQUENCE.steps.len() {
        debug!("Tutorial at step {}", i);
    } else {
        info!("Pre-scripted tutorial sequence completed");
    }

    Ok(())
}

/// Returns the next tutorial action the AI opponent player should take in the
/// tutorial game
pub fn current_opponent_action(game: &GameState) -> Result<GameAction> {
    if let Some(TutorialStep::OpponentAction(a)) =
        crate::SEQUENCE.steps.get(game.data.tutorial_state.index)
    {
        return to_game_action(game, a);
    }

    for i in (0..=game.data.tutorial_state.index).rev() {
        if let Some(TutorialStep::DefaultOpponentAction(tutorial_action)) =
            crate::SEQUENCE.steps.get(i)
        {
            return to_game_action(game, tutorial_action);
        };
    }

    fail!("No opponent action found for index {:?}!", game.data.tutorial_state.index);
}

fn add_game_modifiers(game: &mut GameState, card_names: &[CardName]) -> Result<()> {
    for card_name in card_names {
        mutations::create_and_add_card(game, *card_name, CardPosition::GameModifier)?;
    }

    Ok(())
}

fn remove_game_modifiers(game: &mut GameState, card_names: &[CardName]) -> Result<()> {
    for card_name in card_names {
        let side = rules::get(*card_name).side;
        let card_id = game
            .game_modifiers(side)
            .find(|c| c.name == *card_name)
            .with_error(|| "Card not found")?
            .id;
        mutations::overwrite_card(game, card_id, CardName::OverlordEmptyModifier)?;
    }

    Ok(())
}

fn handle_triggered_action(game: &mut GameState, action: Option<&GameAction>) -> Result<()> {
    for message in
        crate::SEQUENCE.messages.iter().filter(|t| !game.data.tutorial_state.data.has_seen(t.key))
    {
        if trigger_matches(game, &message.trigger, action)? {
            debug!(?message.key, "Triggered tutorial message");
            game.data.tutorial_state.display.append(&mut message.display.clone());
            game.data.tutorial_state.data.mark_seen(message.key);
            break;
        }
    }

    Ok(())
}

fn keep_opening_hand(game: &mut GameState, side: Side) -> Result<()> {
    actions::handle_game_action(
        game,
        side,
        GameAction::PromptAction(PromptAction::MulliganDecision(MulliganDecision::Keep)),
    )
}

fn set_hand(game: &mut GameState, side: Side, cards: &[CardName]) -> Result<()> {
    let hand = game.card_list_for_position(side, CardPosition::Hand(side));
    mutations::shuffle_into_deck(game, side, &hand)?;

    for name in cards {
        let card_id = find_in_deck(game, side, *name)?;
        mutations::move_card(game, card_id, CardPosition::Hand(side))?;
        game.card_mut(card_id).set_revealed_to(side, true);
    }

    // Ignore game update caused by reshuffling
    game.updates.steps.clear();
    Ok(())
}

fn set_top_of_deck(game: &mut GameState, side: Side, cards: &[CardName]) -> Result<()> {
    for name in cards {
        let card_id = find_in_deck(game, side, *name)?;
        mutations::move_card(game, card_id, CardPosition::DeckTop(side))?;
    }

    Ok(())
}

fn to_game_action(game: &GameState, action: &TutorialOpponentAction) -> Result<GameAction> {
    Ok(match action {
        TutorialOpponentAction::DrawCard => GameAction::DrawCard,
        TutorialOpponentAction::PlayCard(name, target) => {
            let card_id = game
                .hand(crate::OPPONENT_SIDE)
                .find(|c| c.name == *name)
                .with_error(|| "Card not found")?
                .id;
            GameAction::PlayCard(card_id, *target)
        }
        TutorialOpponentAction::GainMana => GameAction::GainMana,
        TutorialOpponentAction::InitiateRaid(room_id) => GameAction::InitiateRaid(*room_id),
        TutorialOpponentAction::LevelUpRoom(room_id) => GameAction::LevelUpRoom(*room_id),
        TutorialOpponentAction::UseWeapon { weapon, target } => {
            let weapon =
                game.weapons().find(|c| c.name == *weapon).with_error(|| "Weapon not found")?;
            let target =
                game.minions().find(|c| c.name == *target).with_error(|| "Target not found")?;

            GameAction::PromptAction(PromptAction::EncounterAction(
                EncounterAction::UseWeaponAbility(weapon.id, target.id),
            ))
        }
        TutorialOpponentAction::ScoreAccessedCard(name) => {
            let id = game
                .cards(crate::OPPONENT_SIDE)
                .iter()
                .filter(|c| matches!(c.position(), CardPosition::Room(_, RoomLocation::Occupant)))
                .find(|c| c.name == *name)
                .with_error(|| "Scheme not found")?
                .id;
            GameAction::PromptAction(PromptAction::AccessPhaseAction(AccessPhaseAction::ScoreCard(
                id,
            )))
        }
        TutorialOpponentAction::EndRaid => {
            GameAction::PromptAction(PromptAction::AccessPhaseAction(AccessPhaseAction::EndRaid))
        }
    })
}

/// Wait for an opponent action. Returns true if the provided [GameAction]
/// matches the expected opponent [TutorialOpponentAction].
fn match_opponent_action(
    game: &mut GameState,
    game_action: Option<&GameAction>,
    opponent_action: &TutorialOpponentAction,
) -> Result<bool> {
    debug!(?opponent_action, ?game_action, "Matched expected opponent action");
    trigger_matches(game, &to_trigger(opponent_action), game_action)
}

/// Wait for the player to take specific game actions. Returns true if all
/// named actions have been taken.
fn await_player_actions(
    game: &mut GameState,
    game_action: Option<&GameAction>,
    to_match: &[TutorialTrigger],
) -> Result<bool> {
    let seen = &game.data.tutorial_state.action_indices_seen;

    for (i, tutorial_action) in to_match.iter().enumerate() {
        if game.data.tutorial_state.action_indices_seen.contains(&i) {
            continue;
        }

        let matched = trigger_matches(game, tutorial_action, game_action)?;
        if matched {
            debug!(?seen, ?tutorial_action, ?game_action, "Matched expected player action");
            game.data.tutorial_state.action_indices_seen.insert(i);
            break;
        }
    }

    if game.data.tutorial_state.action_indices_seen.len() == to_match.len() {
        game.data.tutorial_state.action_indices_seen.clear();
        Ok(true)
    } else {
        Ok(false)
    }
}

fn trigger_matches(
    game: &GameState,
    tutorial_action: &TutorialTrigger,
    user_action: Option<&GameAction>,
) -> Result<bool> {
    let Some(action) = user_action else {
        return Ok(false);
    };

    Ok(match (tutorial_action, action) {
        (TutorialTrigger::DrawCardAction, GameAction::DrawCard) => true,
        (TutorialTrigger::PlayAnyCard, GameAction::PlayCard(_, _)) => true,
        (TutorialTrigger::PlayCard(name, t1), GameAction::PlayCard(id, t2)) => {
            game.card(*id).name == *name && t1 == t2
        }
        (TutorialTrigger::GainManaAction, GameAction::GainMana) => true,
        (TutorialTrigger::InitiateRaid(r1), GameAction::InitiateRaid(r2)) => r1 == r2,
        (TutorialTrigger::LevelUpRoom(r1), GameAction::LevelUpRoom(r2)) => r1 == r2,
        (
            TutorialTrigger::UseWeapon { weapon, target },
            GameAction::PromptAction(PromptAction::EncounterAction(
                EncounterAction::UseWeaponAbility(source_id, target_id),
            )),
        ) => game.card(*source_id).name == *weapon && game.card(*target_id).name == *target,
        (
            TutorialTrigger::UseNoWeapon,
            GameAction::PromptAction(PromptAction::EncounterAction(EncounterAction::NoWeapon)),
        ) => true,
        (
            TutorialTrigger::ScoreAccessedCard(name),
            GameAction::PromptAction(PromptAction::AccessPhaseAction(
                AccessPhaseAction::ScoreCard(card_id),
            )),
        ) => game.card(*card_id).name == *name,
        (
            TutorialTrigger::SuccessfullyEndRaid,
            GameAction::PromptAction(PromptAction::AccessPhaseAction(AccessPhaseAction::EndRaid)),
        ) => true,
        _ => false,
    })
}

fn game_state_matches(game: &GameState, trigger: &TutorialGameStateTrigger) -> bool {
    match trigger {
        TutorialGameStateTrigger::HandContainsCard(side, card_name) => {
            game.hand(*side).any(|c| c.name == *card_name)
        }
    }
}

fn to_trigger(opponent_action: &TutorialOpponentAction) -> TutorialTrigger {
    match opponent_action {
        TutorialOpponentAction::DrawCard => TutorialTrigger::DrawCardAction,
        TutorialOpponentAction::PlayCard(name, target) => TutorialTrigger::PlayCard(*name, *target),
        TutorialOpponentAction::GainMana => TutorialTrigger::GainManaAction,
        TutorialOpponentAction::InitiateRaid(room_id) => TutorialTrigger::InitiateRaid(*room_id),
        TutorialOpponentAction::LevelUpRoom(room_id) => TutorialTrigger::LevelUpRoom(*room_id),
        TutorialOpponentAction::UseWeapon { weapon, target } => {
            TutorialTrigger::UseWeapon { weapon: *weapon, target: *target }
        }
        TutorialOpponentAction::ScoreAccessedCard(card_name) => {
            TutorialTrigger::ScoreAccessedCard(*card_name)
        }
        TutorialOpponentAction::EndRaid => TutorialTrigger::SuccessfullyEndRaid,
    }
}

fn display(game: &mut GameState, mut displays: Vec<TutorialDisplay>) -> Result<()> {
    game.data.tutorial_state.display.append(&mut displays);
    Ok(())
}

/// Finds a card with the given `name` in the `side` player's deck, or returns
/// an error if no such card exists.
fn find_in_deck(game: &mut GameState, side: Side, name: CardName) -> Result<CardId> {
    let mut deck = game.cards(side).iter().filter(|c| c.position().in_deck_unknown());
    Ok(deck.find(|c| c.name == name).with_error(|| "Card not found")?.id)
}
