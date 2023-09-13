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
use game_data::card_definition::CardDefinition;
use game_data::card_state::{CardPosition, CardState};
use game_data::delegates::{CardPlayed, CastCardEvent};
use game_data::game::{GameState, HistoryEntry, HistoryEvent};
use game_data::game_actions::{
    ButtonPrompt, CardTarget, GamePrompt, PromptChoice, PromptChoiceLabel, PromptContext,
};
use game_data::game_effect::GameEffect;
use game_data::primitives::{CardId, CardSubtype, CardType, Side};
use game_data::updates::GameUpdate;
use with_error::{verify, WithError};

use crate::mana::ManaPurpose;
use crate::{dispatch, flags, mana, mutations, queries};

/// Puts a card into play.
///
/// Does not validate the legality of taking the play card action.
pub fn run(game: &mut GameState, card_id: CardId, target: CardTarget) -> Result<()> {
    let definition = crate::card_definition(game, card_id);
    if check_play_card_prompts(game, card_id.side, definition, card_id, target) {
        // User needs to make a choice before this card can be played.
        return Ok(());
    }

    mutations::move_card(game, card_id, CardPosition::Played(card_id.side, target))?;

    let actions = queries::action_cost(game, card_id);
    mutations::spend_action_points(game, card_id.side, actions)?;

    if let Some(GamePrompt::PlayCardBrowser(prompt)) = game.player(card_id.side).prompt_queue.get(0)
    {
        // Clear the current 'play card' prompt if one is present.
        verify!(prompt.cards.contains(&card_id), "Unexpected prompt card");
        game.player_mut(card_id.side).prompt_queue.remove(0);
    }

    if flags::enters_play_face_up(game, card_id) {
        let amount = queries::mana_cost(game, card_id).with_error(|| "Card has no mana cost")?;
        mana::spend(game, card_id.side, ManaPurpose::PayForCard(card_id), amount)?;
        if let Some(custom_cost) = &definition.cost.custom_cost {
            (custom_cost.pay)(game, card_id)?;
        }
        game.card_mut(card_id).turn_face_up();
        game.record_update(|| GameUpdate::PlayCardFaceUp(card_id.side, card_id));
    }

    dispatch::invoke_event(game, CastCardEvent(CardPlayed { card_id, target }))?;
    mutations::move_card(
        game,
        card_id,
        queries::played_position(game, card_id.side, card_id, target)?,
    )?;

    game.history
        .push(HistoryEntry { turn: game.info.turn, event: HistoryEvent::PlayedCard(card_id) });
    Ok(())
}

/// Checks whether there are outstanding user choices required by the
/// `user_side` player in order to play the `card_id` card with the provided
/// target. If such a choice needs to be made, writes a UI prompt to `game` and
/// returns `true`. Otherwise, returns `false`.
fn check_play_card_prompts(
    game: &mut GameState,
    user_side: Side,
    definition: &CardDefinition,
    card_id: CardId,
    target: CardTarget,
) -> bool {
    let prompt = match target {
        CardTarget::None => match definition.card_type {
            CardType::Artifact
                if definition.subtypes.contains(&CardSubtype::Weapon)
                    && game_weapons(game).count() >= game_constants::MAXIMUM_WEAPONS_IN_PLAY =>
            {
                Some(card_limit_prompt(
                    game_weapons(game),
                    card_id,
                    target,
                    PromptContext::CardLimit(CardType::Artifact, Some(CardSubtype::Weapon)),
                ))
            }
            CardType::Artifact
                if game.artifacts().count() >= game_constants::MAXIMUM_ARTIFACTS_IN_PLAY =>
            {
                Some(card_limit_prompt(
                    game.artifacts(),
                    card_id,
                    target,
                    PromptContext::CardLimit(CardType::Artifact, None),
                ))
            }
            CardType::Evocation
                if game.evocations().count() >= game_constants::MAXIMUM_EVOCATIONS_IN_PLAY =>
            {
                Some(card_limit_prompt(
                    game.evocations(),
                    card_id,
                    target,
                    PromptContext::CardLimit(CardType::Evocation, None),
                ))
            }
            CardType::Ally if game.allies().count() >= game_constants::MAXIMUM_ALLIES_IN_PLAY => {
                Some(card_limit_prompt(
                    game.allies(),
                    card_id,
                    target,
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
                    card_id,
                    target,
                    PromptContext::CardLimit(CardType::Minion, None),
                ))
            }
            CardType::Project | CardType::Scheme
                if game.occupants(room_id).count() >= game_constants::MAXIMUM_OCCUPANTS_IN_ROOM =>
            {
                Some(card_limit_prompt(
                    game.occupants(room_id),
                    card_id,
                    target,
                    PromptContext::CardLimit(definition.card_type, None),
                ))
            }
            _ => None,
        },
    };

    if let Some(p) = prompt {
        game.player_mut(user_side).prompt_queue.push(p);
        true
    } else {
        false
    }
}

fn game_weapons(game: &GameState) -> impl Iterator<Item = &CardState> {
    game.artifacts().filter(|card| {
        crate::card_definition(game, card.id).subtypes.contains(&CardSubtype::Weapon)
    })
}

fn card_limit_prompt<'a>(
    cards: impl Iterator<Item = &'a CardState>,
    card_id: CardId,
    target: CardTarget,
    context: PromptContext,
) -> GamePrompt {
    GamePrompt::ButtonPrompt(ButtonPrompt {
        context: Some(context),
        choices: cards
            .map(|existing| PromptChoice {
                effects: vec![
                    GameEffect::SacrificeCard(existing.id),
                    GameEffect::PlayCard(card_id, target),
                ],
                anchor_card: Some(existing.id),
                custom_label: Some(PromptChoiceLabel::Sacrifice),
            })
            .chain(iter::once(PromptChoice::from_effect(GameEffect::Cancel)))
            .collect(),
    })
}
