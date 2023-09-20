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

use std::iter;

use anyhow::Result;
use constants::game_constants;
use game_data::action_data::{ActionData, PlayCardData, PlayCardStep};
use game_data::card_state::{CardPosition, CardState};
use game_data::delegates::{CardPlayed, CastCardEvent};
use game_data::game::{GamePhase, GameState, HistoryEntry, HistoryEvent};
use game_data::game_actions::{
    ButtonPrompt, CardTarget, GamePrompt, PromptChoice, PromptChoiceLabel, PromptContext,
};
use game_data::game_effect::GameEffect;
use game_data::game_updates::GameUpdate;
use game_data::primitives::{CardId, CardSubtype, CardType};
use with_error::{verify, WithError};

use crate::mana::ManaPurpose;
use crate::{dispatch, flags, mana, mutations, queries};

/// Starts a new play card action, either as a result the explicit game action
/// or as an effect of another card.
pub fn initiate(game: &mut GameState, card_id: CardId, target: CardTarget) -> Result<()> {
    verify!(game.current_action.is_none(), "An action is already being resolved!");
    game.current_action = Some(ActionData::PlayCard(PlayCardData {
        card_id,
        original_position: game.card(card_id).position(),
        target,
        step: PlayCardStep::Begin,
    }));

    run(game)
}

/// Run the play card state machine, if needed.
///
/// This will advance the state machine through its steps. The state machine
/// pauses if a player is presented with a prompt to respond to, and aborts if
/// the action is aborted. If no play action action is currently active or the
/// state machine cannot currently advance, this function silently ignores the
/// run request.
pub fn run(game: &mut GameState) -> Result<()> {
    loop {
        if has_non_play_prompt(&game.overlord.prompt_queue)
            || has_non_play_prompt(&game.champion.prompt_queue)
        {
            // We pause the state machine if a player has a prompt. We do *not* pause for
            // the PlayCardBrowser prompt since this would prevent anyone from
            // being able to play cards from that browser.
            break;
        }

        if game.info.phase != GamePhase::Play {
            break;
        }

        if let Some(ActionData::PlayCard(play_card)) = game.current_action {
            let step = evaluate_play_step(game, play_card)?;
            if let Some(ActionData::PlayCard(play)) = &mut game.current_action {
                play.step = step;
            }
        } else {
            break;
        }
    }
    Ok(())
}

/// Returns true if the provided prompt queue currently contains a prompt which
/// is *not* the PlayCardBrowser prompt.
fn has_non_play_prompt(queue: &[GamePrompt]) -> bool {
    if !queue.is_empty() {
        !matches!(queue.get(0), Some(GamePrompt::PlayCardBrowser(_)))
    } else {
        false
    }
}

fn evaluate_play_step(game: &mut GameState, play_card: PlayCardData) -> Result<PlayCardStep> {
    match play_card.step {
        PlayCardStep::Begin => Ok(PlayCardStep::CheckLimits),
        PlayCardStep::CheckLimits => check_limits(game, play_card),
        PlayCardStep::MoveToPlayedPosition => move_to_played_position(game, play_card),
        PlayCardStep::PayActionPoints => pay_action_points(game, play_card),
        PlayCardStep::ClearBrowser => clear_browser(game, play_card),
        PlayCardStep::PayManaCost => pay_mana_cost(game, play_card),
        PlayCardStep::PayCustomCost => pay_custom_cost(game, play_card),
        PlayCardStep::TurnFaceUp => turn_face_up(game, play_card),
        PlayCardStep::MoveToTargetPosition => move_to_target_position(game, play_card),
        PlayCardStep::Finish => finish(game, play_card),
    }
}

fn check_limits(game: &mut GameState, play_card: PlayCardData) -> Result<PlayCardStep> {
    let definition = crate::card_definition(game, play_card.card_id);
    let prompt = match play_card.target {
        CardTarget::None => match definition.card_type {
            CardType::Artifact
                if definition.subtypes.contains(&CardSubtype::Weapon)
                    && game_weapons(game).count() >= game_constants::MAXIMUM_WEAPONS_IN_PLAY =>
            {
                Some(card_limit_prompt(
                    game_weapons(game),
                    PromptContext::CardLimit(CardType::Artifact, Some(CardSubtype::Weapon)),
                ))
            }
            CardType::Artifact
                if game.artifacts().count() >= game_constants::MAXIMUM_ARTIFACTS_IN_PLAY =>
            {
                Some(card_limit_prompt(
                    game.artifacts(),
                    PromptContext::CardLimit(CardType::Artifact, None),
                ))
            }
            CardType::Evocation
                if game.evocations().count() >= game_constants::MAXIMUM_EVOCATIONS_IN_PLAY =>
            {
                Some(card_limit_prompt(
                    game.evocations(),
                    PromptContext::CardLimit(CardType::Evocation, None),
                ))
            }
            CardType::Ally if game.allies().count() >= game_constants::MAXIMUM_ALLIES_IN_PLAY => {
                Some(card_limit_prompt(
                    game.allies(),
                    PromptContext::CardLimit(CardType::Ally, None),
                ))
            }
            _ => None,
        },
        CardTarget::Room(room_id) => match definition.card_type {
            CardType::Minion
                if game.defenders_unordered(room_id).count()
                    >= game_constants::MAXIMUM_MINIONS_IN_ROOM =>
            {
                Some(card_limit_prompt(
                    game.defenders_unordered(room_id),
                    PromptContext::CardLimit(CardType::Minion, None),
                ))
            }
            CardType::Project | CardType::Scheme
                if game.occupants(room_id).count() >= game_constants::MAXIMUM_OCCUPANTS_IN_ROOM =>
            {
                Some(card_limit_prompt(
                    game.occupants(room_id),
                    PromptContext::CardLimit(definition.card_type, None),
                ))
            }
            _ => None,
        },
    };

    if let Some(p) = prompt {
        game.player_mut(play_card.card_id.side).prompt_queue.push(p);
    }

    Ok(PlayCardStep::MoveToPlayedPosition)
}

fn move_to_played_position(game: &mut GameState, play_card: PlayCardData) -> Result<PlayCardStep> {
    mutations::move_card(
        game,
        play_card.card_id,
        CardPosition::Played(play_card.card_id.side, play_card.target),
    )?;
    Ok(PlayCardStep::PayActionPoints)
}

fn pay_action_points(game: &mut GameState, play_card: PlayCardData) -> Result<PlayCardStep> {
    let actions = queries::action_cost(game, play_card.card_id);
    mutations::spend_action_points(game, play_card.card_id.side, actions)?;
    Ok(PlayCardStep::ClearBrowser)
}

fn clear_browser(game: &mut GameState, play_card: PlayCardData) -> Result<PlayCardStep> {
    if let Some(GamePrompt::PlayCardBrowser(prompt)) =
        game.player(play_card.card_id.side).prompt_queue.get(0)
    {
        // Clear the current 'play card' prompt if one is present.
        verify!(prompt.cards.contains(&play_card.card_id), "Unexpected prompt card");
        game.player_mut(play_card.card_id.side).prompt_queue.remove(0);
    }

    Ok(PlayCardStep::PayManaCost)
}

fn pay_mana_cost(game: &mut GameState, play_card: PlayCardData) -> Result<PlayCardStep> {
    if flags::enters_play_face_up(game, play_card.card_id) {
        let amount =
            queries::mana_cost(game, play_card.card_id).with_error(|| "Card has no mana cost")?;
        mana::spend(
            game,
            play_card.card_id.side,
            ManaPurpose::PayForCard(play_card.card_id),
            amount,
        )?;

        Ok(PlayCardStep::PayCustomCost)
    } else {
        Ok(PlayCardStep::MoveToTargetPosition)
    }
}

fn pay_custom_cost(game: &mut GameState, play_card: PlayCardData) -> Result<PlayCardStep> {
    let definition = crate::card_definition(game, play_card.card_id);
    if let Some(custom_cost) = &definition.cost.custom_cost {
        (custom_cost.pay)(game, play_card.card_id)?;
    }
    Ok(PlayCardStep::TurnFaceUp)
}

fn turn_face_up(game: &mut GameState, play_card: PlayCardData) -> Result<PlayCardStep> {
    mutations::turn_face_up(game, play_card.card_id);
    game.record_update(|| GameUpdate::PlayCard(play_card.card_id.side, play_card.card_id));
    Ok(PlayCardStep::MoveToTargetPosition)
}

fn move_to_target_position(game: &mut GameState, play_card: PlayCardData) -> Result<PlayCardStep> {
    mutations::move_card(
        game,
        play_card.card_id,
        queries::played_position(
            game,
            play_card.card_id.side,
            play_card.card_id,
            play_card.target,
        )?,
    )?;
    Ok(PlayCardStep::Finish)
}

fn finish(game: &mut GameState, play_card: PlayCardData) -> Result<PlayCardStep> {
    game.history.push(HistoryEntry {
        turn: game.info.turn,
        event: HistoryEvent::PlayedCard(play_card.card_id),
    });

    game.current_action = None;

    dispatch::invoke_event(
        game,
        CastCardEvent(CardPlayed { card_id: play_card.card_id, target: play_card.target }),
    )?;

    Ok(PlayCardStep::Finish)
}

fn game_weapons(game: &GameState) -> impl Iterator<Item = &CardState> {
    game.artifacts().filter(|card| {
        crate::card_definition(game, card.id).subtypes.contains(&CardSubtype::Weapon)
    })
}

fn card_limit_prompt<'a>(
    cards: impl Iterator<Item = &'a CardState>,
    context: PromptContext,
) -> GamePrompt {
    GamePrompt::ButtonPrompt(ButtonPrompt {
        context: Some(context),
        choices: cards
            .map(|existing| PromptChoice {
                effects: vec![GameEffect::SacrificeCard(existing.id)],
                anchor_card: Some(existing.id),
                custom_label: Some(PromptChoiceLabel::Sacrifice),
            })
            .chain(iter::once(PromptChoice::from_effect(GameEffect::AbortCurrentGameAction)))
            .collect(),
    })
}
