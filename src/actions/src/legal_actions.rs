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

//! Identifies legal game actions for a given game state.

use std::iter;

use anyhow::Result;
use game_data::card_definition::{AbilityType, TargetRequirement};
use game_data::game_actions::{
    CardTarget, CardTargetKind, GameAction, GamePrompt, GameStateAction, PromptAction,
};
use game_data::game_state::{GamePhase, GameState, MulliganDecision};
use game_data::primitives::{AbilityId, CardId, RoomId, Side};
use raid_state::raid_prompt;
use rules::{flags, queries, CardDefinitionExt};
use with_error::fail;

/// Returns an iterator over currently-legal [GameAction]s for the `side` player
/// in the given [GameState].
pub fn evaluate<'a>(
    game: &'a GameState,
    side: Side,
) -> Result<Box<dyn Iterator<Item = GameAction> + 'a>> {
    match &game.info.phase {
        GamePhase::ResolveMulligans(data) => {
            return Ok(if data.decision(side).is_some() {
                fail!("Error: Mulligan decision already submitted")
            } else {
                Box::new(
                    iter::once(GameAction::GameStateAction(GameStateAction::MulliganDecision(
                        MulliganDecision::Keep,
                    )))
                    .chain(iter::once(GameAction::GameStateAction(
                        GameStateAction::MulliganDecision(MulliganDecision::Mulligan),
                    ))),
                )
            });
        }
        GamePhase::Play => {}
        GamePhase::GameOver { .. } => fail!("Game has ended"),
    }

    if let Some(prompt) = &game.player(side).prompt_queue.get(0) {
        if let GamePrompt::ButtonPrompt(buttons) = prompt {
            return Ok(Box::new(
                buttons
                    .choices
                    .iter()
                    .enumerate()
                    .map(|(i, _)| GameAction::PromptAction(PromptAction::ButtonPromptSelect(i))),
            ));
        } else {
            todo!("Implement support for browser prompts");
        }
    }

    if game.raid.is_some() {
        return Ok(Box::new(raid_prompt::legal_actions(game, side).into_iter()));
    }

    if flags::in_main_phase_with_action_point(game, side) {
        Ok(Box::new(
            enum_iterator::all::<RoomId>()
                .filter(move |room_id| flags::can_take_initiate_raid_action(game, side, *room_id))
                .map(GameAction::InitiateRaid)
                .chain(
                    flags::can_take_end_turn_action(game, side)
                        .then_some(GameAction::GameStateAction(GameStateAction::EndTurnAction)),
                )
                .chain(
                    enum_iterator::all::<RoomId>()
                        .filter(move |room_id| {
                            flags::can_take_level_up_room_action(game, side, *room_id)
                        })
                        .map(GameAction::LevelUpRoom),
                )
                .chain(game.hand(side).flat_map(move |c| legal_card_actions(game, side, c.id)))
                .chain(flags::can_take_draw_card_action(game, side).then_some(GameAction::DrawCard))
                .chain(
                    flags::can_take_gain_mana_action(game, side).then_some(GameAction::GainMana),
                ),
        ))
    } else if flags::can_take_end_turn_action(game, side) {
        Ok(Box::new(iter::once(GameAction::GameStateAction(GameStateAction::EndTurnAction))))
    } else if flags::can_take_start_turn_action(game, side) {
        Ok(Box::new(iter::once(GameAction::GameStateAction(GameStateAction::StartTurnAction))))
    } else {
        fail!("Error: player cannot currently act")
    }
}

/// Builds an iterator over all possible 'play card' and 'activate ability'
/// actions for the provided card.
fn legal_card_actions(
    game: &GameState,
    side: Side,
    card_id: CardId,
) -> impl Iterator<Item = GameAction> + '_ {
    let target_kind = queries::card_target_kind(game, card_id);

    // Iterator combining pattern suggested by *the* Niko Matsakis
    // https://stackoverflow.com/a/52064434/298036
    let play_in_room = if target_kind == CardTargetKind::Room {
        Some(enum_iterator::all::<RoomId>().filter_map(move |room_id| {
            if flags::can_take_play_card_action(game, side, card_id, CardTarget::Room(room_id)) {
                Some(GameAction::PlayCard(card_id, CardTarget::Room(room_id)))
            } else {
                None
            }
        }))
    } else {
        None
    };

    let play_card = if target_kind == CardTargetKind::None
        && flags::can_take_play_card_action(game, side, card_id, CardTarget::None)
    {
        Some(iter::once(GameAction::PlayCard(card_id, CardTarget::None)))
    } else {
        None
    };

    let activated = game
        .card(card_id)
        .definition()
        .ability_ids(card_id)
        .flat_map(move |ability_id| legal_ability_actions(game, side, ability_id));

    play_in_room.into_iter().flatten().chain(play_card.into_iter().flatten()).chain(activated)
}

/// Builds an iterator over all possible 'activate ability' actions for the
/// provided card.
fn legal_ability_actions(
    game: &GameState,
    side: Side,
    ability_id: AbilityId,
) -> impl Iterator<Item = GameAction> + '_ {
    let ability = rules::ability_definition(game, ability_id);
    let mut activate = None;
    let mut target_rooms = None;

    if let AbilityType::Activated(_, targeting) = &ability.ability_type {
        match targeting {
            TargetRequirement::None => {
                if flags::can_take_activate_ability_action(game, side, ability_id, CardTarget::None)
                {
                    activate =
                        Some(iter::once(GameAction::ActivateAbility(ability_id, CardTarget::None)))
                }
            }
            TargetRequirement::TargetRoom(_) => {
                target_rooms = Some(enum_iterator::all::<RoomId>().filter_map(move |room_id| {
                    if flags::can_take_activate_ability_action(
                        game,
                        side,
                        ability_id,
                        CardTarget::Room(room_id),
                    ) {
                        Some(GameAction::ActivateAbility(ability_id, CardTarget::Room(room_id)))
                    } else {
                        None
                    }
                }))
            }
        }
    }

    activate.into_iter().flatten().chain(target_rooms.into_iter().flatten())
}
