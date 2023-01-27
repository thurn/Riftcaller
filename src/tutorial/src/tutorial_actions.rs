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
use data::card_name::CardName;
use data::card_state::CardPosition;
use data::game::{GameState, MulliganDecision};
use data::game_actions::{AccessPhaseAction, EncounterAction, GameAction, PromptAction};
use data::primitives::{CardId, RoomLocation, Side};
use data::tutorial_data::{TutorialDisplay, TutorialOpponentAction, TutorialStep, TutorialTrigger};
use rules::mutations;
use tracing::{debug, debug_span};
use with_error::WithError;

/// Handle applying tutorial actions
pub fn handle_tutorial_action(
    game: &mut GameState,
    mut user_action: Option<GameAction>,
) -> Result<()> {
    let _span = debug_span!("handle_tutorial_actions").entered();
    let mut i = game.data.tutorial_state.index;
    game.data.tutorial_state.display.clear();

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
            TutorialStep::AwaitPlayerActions(actions) => {
                if await_player_actions(game, user_action, actions)? {
                    user_action = None; // Consume action, avoid matching again
                } else {
                    debug!(?actions, "Awaiting user action");
                    break;
                }
                Ok(())
            }
            TutorialStep::Display(displays) => display(game, displays.clone()),
        }?;

        i += 1;
    }

    game.data.tutorial_state.index = i;
    debug!("Tutorial at step {}", i);

    Ok(())
}

/// Returns the next tutorial action the AI opponent player should take in the
/// current game state, if any.
pub fn current_opponent_action(game: &GameState) -> Result<Option<GameAction>> {
    let Some(TutorialStep::OpponentAction(tutorial_action)) =
        crate::SEQUENCE.steps.get(game.data.tutorial_state.index) else {
        return Ok(None)
    };

    Ok(Some(to_game_action(game, tutorial_action)?))
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
    game_action: Option<GameAction>,
    opponent_action: &TutorialOpponentAction,
) -> Result<bool> {
    let Some(user_action) = game_action else {
        return Ok(false);
    };

    debug!(?opponent_action, ?user_action, "Matched expected opponent action");
    actions_match(game, &to_trigger(opponent_action), &user_action)
}

/// Wait for the player to take specific game actions. Returns true if all
/// named actions have been taken.
fn await_player_actions(
    game: &mut GameState,
    game_action: Option<GameAction>,
    to_match: &[TutorialTrigger],
) -> Result<bool> {
    let seen = &game.data.tutorial_state.seen;

    let Some(user_action) = game_action else {
        return Ok(false);
    };

    for (i, tutorial_action) in to_match.iter().enumerate() {
        if game.data.tutorial_state.seen.contains(&i) {
            continue;
        }

        let matched = actions_match(game, tutorial_action, &user_action)?;
        if matched {
            debug!(?seen, ?tutorial_action, ?user_action, "Matched expected player action");
            game.data.tutorial_state.seen.insert(i);
            break;
        }
    }

    if game.data.tutorial_state.seen.len() == to_match.len() {
        debug!("Matched all expected tutorial user actions");
        game.data.tutorial_state.seen.clear();
        Ok(true)
    } else {
        Ok(false)
    }
}

fn actions_match(
    game: &GameState,
    tutorial_action: &TutorialTrigger,
    user_action: &GameAction,
) -> Result<bool> {
    Ok(match (tutorial_action, user_action) {
        (TutorialTrigger::DrawCard, GameAction::DrawCard) => true,
        (TutorialTrigger::PlayAnyCard, GameAction::PlayCard(_, _)) => true,
        (TutorialTrigger::PlayCard(name, t1), GameAction::PlayCard(id, t2)) => {
            game.card(*id).name == *name && t1 == t2
        }
        (TutorialTrigger::GainMana, GameAction::GainMana) => true,
        (TutorialTrigger::InitiateRaid(r1), GameAction::InitiateRaid(r2)) => r1 == r2,
        (TutorialTrigger::LevelUpRoom(r1), GameAction::LevelUpRoom(r2)) => r1 == r2,
        (
            TutorialTrigger::UseWeapon { weapon, target },
            GameAction::PromptAction(PromptAction::EncounterAction(
                EncounterAction::UseWeaponAbility(source_id, target_id),
            )),
        ) => game.card(*source_id).name == *weapon && game.card(*target_id).name == *target,
        (
            TutorialTrigger::ScoreAccessedCard(name),
            GameAction::PromptAction(PromptAction::AccessPhaseAction(
                AccessPhaseAction::ScoreCard(card_id),
            )),
        ) => game.card(*card_id).name == *name,
        (
            TutorialTrigger::EndRaid,
            GameAction::PromptAction(PromptAction::AccessPhaseAction(AccessPhaseAction::EndRaid)),
        ) => true,
        _ => false,
    })
}

fn to_trigger(opponent_action: &TutorialOpponentAction) -> TutorialTrigger {
    match opponent_action {
        TutorialOpponentAction::DrawCard => TutorialTrigger::DrawCard,
        TutorialOpponentAction::PlayCard(name, target) => TutorialTrigger::PlayCard(*name, *target),
        TutorialOpponentAction::GainMana => TutorialTrigger::GainMana,
        TutorialOpponentAction::InitiateRaid(room_id) => TutorialTrigger::InitiateRaid(*room_id),
        TutorialOpponentAction::LevelUpRoom(room_id) => TutorialTrigger::LevelUpRoom(*room_id),
        TutorialOpponentAction::UseWeapon { weapon, target } => {
            TutorialTrigger::UseWeapon { weapon: *weapon, target: *target }
        }
        TutorialOpponentAction::ScoreAccessedCard(card_name) => {
            TutorialTrigger::ScoreAccessedCard(*card_name)
        }
        TutorialOpponentAction::EndRaid => TutorialTrigger::EndRaid,
    }
}

fn display(game: &mut GameState, mut displays: Vec<TutorialDisplay>) -> Result<()> {
    game.data.tutorial_state.display.append(&mut displays);
    Ok(())
}

/// Finds a card with the given `name` in the `side` player's deck, or returns
/// an error if no such card exists.
fn find_in_deck(game: &mut GameState, side: Side, name: CardName) -> Result<CardId> {
    Ok(game.deck(side).find(|c| c.name == name).with_error(|| "Card not found")?.id)
}
