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
use core_data::game_primitives::{CardId, CardSubtype, CardType, InitiatedBy, Side};
use game_data::animation_tracker::GameAnimation;
use game_data::card_state::{CardPosition, CardState};
use game_data::delegate_data::{CardPlayed, PlayCardEvent, Scope};
use game_data::game_actions::{
    ButtonPrompt, ButtonPromptContext, CardTarget, GamePrompt, PromptChoice, PromptChoiceLabel,
    UnplayedAction,
};
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::history_data::HistoryEvent;
use game_data::state_machine_data::{PlayCardData, PlayCardOptions, PlayCardStep};
use with_error::{verify, WithError};

use crate::mana::ManaPurpose;
use crate::state_machine::StateMachine;
use crate::{dispatch, flags, mana, mutations, queries, state_machine, CardDefinitionExt};

/// Starts a new play card action, either as a result the explicit game action
/// or as an effect of another card.
pub fn initiate(
    game: &mut GameState,
    card_id: CardId,
    target: CardTarget,
    initiated_by: InitiatedBy,
    options: PlayCardOptions,
) -> Result<()> {
    state_machine::initiate(
        game,
        PlayCardData { card_id, initiated_by, target, options, step: PlayCardStep::Begin },
    )
}

/// Run the play card state machine, if needed.
///
/// This will advance the state machine through its steps. The state machine
/// pauses if a player is presented with a prompt to respond to, and aborts if
/// the action is aborted. If no play action action is currently active or the
/// state machine cannot currently advance, this function silently ignores the
/// run request.
pub fn run(game: &mut GameState) -> Result<()> {
    state_machine::run::<PlayCardData>(game)
}

/// Stops the currently-active 'play card' game action, if any
pub fn abort(game: &mut GameState) {
    game.state_machines.play_card.pop();
}

/// Returns true if the `card_id` card is currently part of the active
/// `play_card` state machine, and this game action was initiated by the card
/// with the provided [Scope].
pub fn currently_being_played_by(game: &GameState, card_id: CardId, scope: Scope) -> bool {
    if let Some(data) = game.state_machines.play_card.last() {
        data.card_id == card_id && data.initiated_by.card_id() == Some(scope.card_id())
    } else {
        false
    }
}

impl StateMachine for PlayCardData {
    type Data = Self;
    type Step = PlayCardStep;

    fn get(game: &GameState) -> &Vec<Self> {
        &game.state_machines.play_card
    }

    fn get_mut(game: &mut GameState) -> &mut Vec<Self> {
        &mut game.state_machines.play_card
    }

    fn step(&self) -> Self::Step {
        self.step
    }

    fn step_mut(&mut self) -> &mut Self::Step {
        &mut self.step
    }

    fn data(&self) -> Self::Data {
        *self
    }

    fn evaluate(
        game: &mut GameState,
        step: PlayCardStep,
        data: PlayCardData,
    ) -> Result<Option<Self::Step>> {
        match step {
            PlayCardStep::Begin => Ok(Some(PlayCardStep::CheckLimits)),
            PlayCardStep::CheckLimits => check_limits(game, data),
            PlayCardStep::ClearPreviousState => clear_previous_state(game, data),
            PlayCardStep::AddToHistory => add_to_history(game, data),
            PlayCardStep::MoveToPlayedPosition => move_to_played_position(game, data),
            PlayCardStep::PayActionPoints => pay_action_points(game, data),
            PlayCardStep::ApplyPlayCardBrowser => apply_play_card_browser(game, data),
            PlayCardStep::PayManaCost => pay_mana_cost(game, data),
            PlayCardStep::PayCustomCost => pay_custom_cost(game, data),
            PlayCardStep::TurnFaceUp => turn_face_up(game, data),
            PlayCardStep::MoveToTargetPosition => move_to_target_position(game, data),
            PlayCardStep::Finish => finish(game, data),
        }
    }
}

fn check_limits(game: &mut GameState, play_card: PlayCardData) -> Result<Option<PlayCardStep>> {
    let definition = game.card(play_card.card_id).definition();
    let prompt = match play_card.target {
        CardTarget::None => match definition.card_type {
            CardType::Artifact
                if definition.subtypes.contains(&CardSubtype::Weapon)
                    && game_weapons(game).count() >= game_constants::MAXIMUM_WEAPONS_IN_PLAY =>
            {
                Some(card_limit_prompt(
                    game_weapons(game),
                    ButtonPromptContext::CardLimit(CardType::Artifact, Some(CardSubtype::Weapon)),
                ))
            }
            CardType::Artifact
                if game.artifacts().count() >= game_constants::MAXIMUM_ARTIFACTS_IN_PLAY =>
            {
                Some(card_limit_prompt(
                    game.artifacts(),
                    ButtonPromptContext::CardLimit(CardType::Artifact, None),
                ))
            }
            CardType::Evocation
                if game.evocations().count() >= game_constants::MAXIMUM_EVOCATIONS_IN_PLAY =>
            {
                Some(card_limit_prompt(
                    game.evocations(),
                    ButtonPromptContext::CardLimit(CardType::Evocation, None),
                ))
            }
            CardType::Ally if game.allies().count() >= game_constants::MAXIMUM_ALLIES_IN_PLAY => {
                Some(card_limit_prompt(
                    game.allies(),
                    ButtonPromptContext::CardLimit(CardType::Ally, None),
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
                    ButtonPromptContext::CardLimit(CardType::Minion, None),
                ))
            }
            CardType::Project | CardType::Scheme
                if game.occupants(room_id).count() >= game_constants::MAXIMUM_OCCUPANTS_IN_ROOM =>
            {
                Some(card_limit_prompt(
                    game.occupants(room_id),
                    ButtonPromptContext::CardLimit(definition.card_type, None),
                ))
            }
            _ => None,
        },
    };

    if let Some(p) = prompt {
        game.player_mut(play_card.card_id.side).prompt_stack.push(p);
    }

    Ok(Some(PlayCardStep::ClearPreviousState))
}

fn clear_previous_state(
    game: &mut GameState,
    play_card: PlayCardData,
) -> Result<Option<PlayCardStep>> {
    game.card_mut(play_card.card_id).clear_played_state();
    Ok(Some(PlayCardStep::AddToHistory))
}

fn add_to_history(game: &mut GameState, play_card: PlayCardData) -> Result<Option<PlayCardStep>> {
    game.add_history_event(HistoryEvent::PlayCard(
        play_card.card_id,
        play_card.target,
        play_card.initiated_by,
    ));
    Ok(Some(PlayCardStep::MoveToPlayedPosition))
}

fn move_to_played_position(
    game: &mut GameState,
    play_card: PlayCardData,
) -> Result<Option<PlayCardStep>> {
    mutations::move_card(
        game,
        play_card.card_id,
        CardPosition::Played(play_card.card_id.side, play_card.target),
    )?;
    Ok(Some(PlayCardStep::PayActionPoints))
}

fn pay_action_points(
    game: &mut GameState,
    play_card: PlayCardData,
) -> Result<Option<PlayCardStep>> {
    if !play_card.options.ignore_action_cost {
        let actions = queries::action_cost(game, play_card.card_id);
        mutations::spend_action_points(game, play_card.card_id.side, actions)?;
    }

    Ok(Some(PlayCardStep::ApplyPlayCardBrowser))
}

fn apply_play_card_browser(
    game: &mut GameState,
    play_card: PlayCardData,
) -> Result<Option<PlayCardStep>> {
    invoke_play_card_browser(game, play_card.card_id.side, Some(play_card.card_id))?;
    Ok(Some(PlayCardStep::PayManaCost))
}

/// Handles resolution of a [GamePrompt] with a `PlayCardBrowser`. Fires the
/// [UnplayedAction] for this browser and clears the user's prompt queue.
pub fn invoke_play_card_browser(
    game: &mut GameState,
    side: Side,
    card_id: Option<CardId>,
) -> Result<()> {
    if let Some(GamePrompt::PlayCardBrowser(prompt)) = game.player(side).prompt_stack.current() {
        if let Some(id) = card_id {
            verify!(prompt.cards.contains(&id), "Unexpected prompt card");
        }

        match prompt.unplayed_action {
            UnplayedAction::None => {}
            UnplayedAction::Discard => {
                let discard = prompt
                    .cards
                    .iter()
                    .copied()
                    .filter(|id| Some(*id) != card_id)
                    .collect::<Vec<_>>();
                for card_id in discard {
                    mutations::discard_card(game, card_id)?;
                }
            }
        }

        game.player_mut(side).prompt_stack.pop();
    }
    Ok(())
}

fn pay_mana_cost(game: &mut GameState, play_card: PlayCardData) -> Result<Option<PlayCardStep>> {
    if flags::enters_play_face_up(game, play_card.card_id) {
        if play_card.options.ignore_mana_cost {
            Ok(Some(PlayCardStep::PayCustomCost))
        } else {
            let amount = queries::mana_cost(game, play_card.card_id)
                .with_error(|| "Card has no mana cost")?;
            mana::spend(
                game,
                play_card.card_id.side,
                InitiatedBy::GameAction,
                ManaPurpose::PayForCard(play_card.card_id),
                amount,
            )?;

            Ok(Some(PlayCardStep::PayCustomCost))
        }
    } else {
        Ok(Some(PlayCardStep::MoveToTargetPosition))
    }
}

fn pay_custom_cost(game: &mut GameState, play_card: PlayCardData) -> Result<Option<PlayCardStep>> {
    let definition = game.card(play_card.card_id).definition();
    if let Some(custom_cost) = &definition.cost.custom_cost {
        (custom_cost.pay)(game, play_card.card_id)?;
    }
    Ok(Some(PlayCardStep::TurnFaceUp))
}

fn turn_face_up(game: &mut GameState, play_card: PlayCardData) -> Result<Option<PlayCardStep>> {
    mutations::turn_face_up(game, play_card.card_id);
    game.add_animation(|| GameAnimation::PlayCard(play_card.card_id.side, play_card.card_id));
    Ok(Some(PlayCardStep::MoveToTargetPosition))
}

fn move_to_target_position(
    game: &mut GameState,
    play_card: PlayCardData,
) -> Result<Option<PlayCardStep>> {
    mutations::move_card(
        game,
        play_card.card_id,
        queries::played_position(game, play_card.card_id.side, play_card.card_id, play_card.target)
            .with_error(|| "No valid position")?,
    )?;
    Ok(Some(PlayCardStep::Finish))
}

fn finish(game: &mut GameState, play_card: PlayCardData) -> Result<Option<PlayCardStep>> {
    dispatch::invoke_event(
        game,
        PlayCardEvent(CardPlayed { card_id: play_card.card_id, target: play_card.target }),
    )?;

    Ok(None)
}

fn game_weapons(game: &GameState) -> impl Iterator<Item = &CardState> {
    game.artifacts()
        .filter(|card| game.card(card.id).definition().subtypes.contains(&CardSubtype::Weapon))
}

fn card_limit_prompt<'a>(
    cards: impl Iterator<Item = &'a CardState>,
    context: ButtonPromptContext,
) -> GamePrompt {
    GamePrompt::ButtonPrompt(ButtonPrompt {
        context: Some(context),
        choices: cards
            .map(|existing| PromptChoice {
                effects: vec![GameEffect::SacrificeCard(existing.id)],
                anchor_card: Some(existing.id),
                custom_label: Some(PromptChoiceLabel::Sacrifice),
            })
            .chain(iter::once(PromptChoice::new().effect(GameEffect::AbortPlayingCard)))
            .collect(),
    })
}
