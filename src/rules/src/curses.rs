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
use game_data::delegate_data::{CursesReceivedEvent, WillReceiveCursesEvent};
use game_data::game_state::{GamePhase, GameState};
use game_data::history_data::HistoryEvent;
use game_data::primitives::{CurseCount, HasAbilityId};
use game_data::state_machines::{GiveCursesData, GiveCursesStep};
use with_error::verify;

use crate::dispatch;

/// Gives curses to the Champion player.
pub fn give_curses(
    game: &mut GameState,
    source: impl HasAbilityId,
    quantity: CurseCount,
) -> Result<()> {
    verify!(game.state_machines.give_curses.is_none(), "Curse is already being resolved!");

    game.state_machines.give_curses =
        Some(GiveCursesData { quantity, source: source.ability_id(), step: GiveCursesStep::Begin });

    run_state_machine(game)
}

/// Remove *up to* `amount` curses from the Champion player.
pub fn remove_curses(game: &mut GameState, amount: CurseCount) -> Result<()> {
    game.champion.curses = game.champion.curses.saturating_sub(amount);
    Ok(())
}

/// Run the deal damage state machine, if needed.
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    loop {
        if !(game.overlord.prompt_stack.is_empty() & game.champion.prompt_stack.is_empty()) {
            break;
        }

        if game.info.phase != GamePhase::Play {
            break;
        }

        if let Some(data) = &game.state_machines.give_curses {
            let step = match data.step {
                GiveCursesStep::Begin => GiveCursesStep::WillReceiveCursesEvent,
                GiveCursesStep::WillReceiveCursesEvent => {
                    dispatch::invoke_event(game, WillReceiveCursesEvent(data.quantity))?;
                    GiveCursesStep::AddCurses
                }
                GiveCursesStep::AddCurses => {
                    game.champion.curses += data.quantity;
                    GiveCursesStep::CursesReceivedEvent
                }
                GiveCursesStep::CursesReceivedEvent => {
                    dispatch::invoke_event(game, CursesReceivedEvent(data.quantity))?;
                    GiveCursesStep::Finish
                }
                GiveCursesStep::Finish => {
                    game.add_history_event(HistoryEvent::GiveCurse(data.quantity));
                    game.state_machines.give_curses = None;
                    GiveCursesStep::Finish
                }
            };

            if let Some(updated) = &mut game.state_machines.give_curses {
                updated.step = step;
            }
        } else {
            break;
        }
    }
    Ok(())
}
