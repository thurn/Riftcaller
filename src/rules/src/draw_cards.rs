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
use core_data::game_primitives::{InitiatedBy, Side};
use game_data::animation_tracker::GameAnimation;
use game_data::card_state::CardPosition;
use game_data::delegate_data::{DrawCardsViaAbilityEvent, WillDrawCardsEvent};
use game_data::game_actions::GamePrompt;
use game_data::game_state::{GamePhase, GameState, PromptStack};
use game_data::state_machines::{DrawCardsData, DrawCardsStep};

use crate::{dispatch, mutations};

/// Function to draw `count` cards from the top of a player's deck and
/// place them into their hand.
///
/// If there are insufficient cards available:
///  - If `side == Overlord`, the Overlord player loses the game and no cards
///    are returned.
///  - If `side == Champion`, all remaining cards are returned.
///
/// Cards are marked as revealed to the `side` player.
pub fn run(game: &mut GameState, side: Side, quantity: u32, source: InitiatedBy) -> Result<()> {
    game.state_machines.draw_cards.push(DrawCardsData {
        side,
        quantity,
        draw_is_prevented: false,
        source,
        step: DrawCardsStep::Begin,
    });

    run_state_machine(game)
}

/// Run the draw cards state machine, if needed.
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    loop {
        if has_blocking_prompt(&game.overlord.prompt_stack)
            || has_blocking_prompt(&game.champion.prompt_stack)
        {
            break;
        }

        if matches!(game.info.phase, GamePhase::GameOver { .. }) {
            break;
        }

        if let Some(data) = game.state_machines.draw_cards.last() {
            let quantity = data.quantity;
            let side = data.side;

            let step = match data.step {
                DrawCardsStep::Begin => Some(DrawCardsStep::WillDrawCardsEvent),
                DrawCardsStep::WillDrawCardsEvent => {
                    dispatch::invoke_event(game, WillDrawCardsEvent(side))?;
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
                    let card_ids = mutations::realize_top_of_deck(game, side, quantity)?;

                    if card_ids.len() != quantity as usize && side == Side::Overlord {
                        mutations::game_over(game, side.opponent())?;
                        None
                    } else {
                        for card_id in &card_ids {
                            mutations::set_visible_to(game, *card_id, side, true);
                        }
                        game.add_animation(|| GameAnimation::DrawCards(side, card_ids.clone()));

                        for card_id in &card_ids {
                            mutations::move_card(game, *card_id, CardPosition::Hand(side))?;
                        }

                        Some(DrawCardsStep::DrawCardsViaAbilityEvent(card_ids.len() as u32))
                    }
                }
                DrawCardsStep::DrawCardsViaAbilityEvent(count) => {
                    if matches!(data.source, InitiatedBy::Ability(..)) {
                        dispatch::invoke_event(game, DrawCardsViaAbilityEvent(side))?;
                    }

                    Some(DrawCardsStep::AddToHistory(count))
                }
                DrawCardsStep::AddToHistory(count) => {
                    let source = data.source;
                    game.current_history_counters(side).cards_drawn += count;
                    if matches!(source, InitiatedBy::Ability(..) | InitiatedBy::SilentAbility(..)) {
                        game.current_history_counters(side).cards_drawn_via_abilities += count;
                    }

                    Some(DrawCardsStep::Finish)
                }
                DrawCardsStep::Finish => None,
            };

            if let Some(s) = step {
                if let Some(updated) = game.state_machines.draw_cards.last_mut() {
                    updated.step = s;
                }
            } else {
                game.state_machines.draw_cards.pop();
            }
        } else {
            break;
        }
    }
    Ok(())
}

/// Returns true if the provided prompt queue currently contains a prompt which
/// should block drawing cards
fn has_blocking_prompt(stack: &PromptStack) -> bool {
    matches!(stack.current(), Some(GamePrompt::ButtonPrompt(..)))
}
