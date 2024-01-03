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

use anyhow::Result;
use core_data::game_primitives::{InitiatedBy, Side};
use dispatcher::dispatch;
use game_data::animation_tracker::GameAnimation;
use game_data::card_state::CardPosition;
use game_data::delegate_data::{DrawCardsViaAbilityEvent, WillDrawCardsEvent};
use game_data::game_state::GameState;
use game_data::state_machine_data::{DrawCardsData, DrawCardsStep};

use crate::mutations::RealizeCards;
use crate::state_machine::StateMachine;
use crate::{mutations, state_machine};

/// Function to draw `count` cards from the top of a player's deck and
/// place them into their hand.
///
/// If there are insufficient cards available:
///  - If `side == Covenant`, the Covenant player loses the game and no cards
///    are returned.
///  - If `side == Riftcaller`, all remaining cards are returned.
///
/// Cards are marked as revealed to the `side` player.
pub fn run(game: &mut GameState, side: Side, quantity: u32, source: InitiatedBy) -> Result<()> {
    state_machine::initiate(
        game,
        DrawCardsData {
            side,
            quantity,
            draw_is_prevented: false,
            source,
            step: DrawCardsStep::Begin,
        },
    )
}

/// Run the draw cards state machine, if needed.
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    state_machine::run::<DrawCardsData>(game)
}

impl StateMachine for DrawCardsData {
    type Step = DrawCardsStep;

    fn get(game: &GameState) -> &Vec<Self> {
        &game.state_machines.draw_cards
    }

    fn get_mut(game: &mut GameState) -> &mut Vec<Self> {
        &mut game.state_machines.draw_cards
    }

    fn step(&self) -> Self::Step {
        self.step
    }

    fn step_mut(&mut self) -> &mut Self::Step {
        &mut self.step
    }

    fn evaluate(
        game: &mut GameState,
        step: DrawCardsStep,
        data: DrawCardsData,
    ) -> Result<Option<DrawCardsStep>> {
        Ok(match step {
            DrawCardsStep::Begin => Some(DrawCardsStep::WillDrawCardsEvent),
            DrawCardsStep::WillDrawCardsEvent => {
                dispatch::invoke_event(game, WillDrawCardsEvent(&data.side))?;
                Some(DrawCardsStep::CheckIfDrawPrevented)
            }
            DrawCardsStep::CheckIfDrawPrevented => {
                if data.draw_is_prevented {
                    None
                } else {
                    Some(DrawCardsStep::DrawCards)
                }
            }
            DrawCardsStep::DrawCards => {
                let card_ids = mutations::realize_top_of_deck(
                    game,
                    data.side,
                    data.quantity,
                    RealizeCards::SetVisibleToOwner,
                )?;

                if card_ids.len() != data.quantity as usize && data.side == Side::Covenant {
                    mutations::game_over(game, data.side.opponent())?;
                    None
                } else {
                    game.add_animation(|| GameAnimation::DrawCards(data.side, card_ids.clone()));

                    for card_id in &card_ids {
                        mutations::move_card(game, *card_id, CardPosition::Hand(data.side))?;
                    }

                    Some(DrawCardsStep::DrawCardsViaAbilityEvent(card_ids.len() as u32))
                }
            }
            DrawCardsStep::DrawCardsViaAbilityEvent(count) => {
                if matches!(data.source, InitiatedBy::Ability(..)) {
                    dispatch::invoke_event(game, DrawCardsViaAbilityEvent(&data.side))?;
                }

                Some(DrawCardsStep::AddToHistory(count))
            }
            DrawCardsStep::AddToHistory(count) => {
                let source = data.source;
                game.current_history_counters(data.side).cards_drawn += count;
                if matches!(source, InitiatedBy::Ability(..) | InitiatedBy::SilentAbility(..)) {
                    game.current_history_counters(data.side).cards_drawn_via_abilities += count;
                }

                Some(DrawCardsStep::Finish)
            }
            DrawCardsStep::Finish => None,
        })
    }
}
