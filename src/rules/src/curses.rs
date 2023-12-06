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
use core_data::game_primitives::{CurseCount, HasAbilityId, Side};
use game_data::delegate_data::{CursesReceivedEvent, WillReceiveCursesEvent};
use game_data::game_state::GameState;
use game_data::state_machine_data::{GiveCurseOptions, GiveCursesData, GiveCursesStep};
use game_data::utils;

use crate::state_machine::StateMachine;
use crate::{dispatch, state_machine};

/// Returns the number of curses the Riftcaller player currently has
pub fn get(game: &GameState) -> CurseCount {
    let mut base = game.riftcaller.curse_state.base_curses;
    if let Some((turn, count)) = game.riftcaller.curse_state.turn_curses {
        if turn == game.info.turn {
            base += count;
        }
    }
    base
}

/// Returns true if the Riftcaller player is currently cursed
pub fn is_riftcaller_cursed(game: &GameState) -> bool {
    get(game) > 0
}

/// Gives curses to the Riftcaller player.
pub fn give_curses(
    game: &mut GameState,
    source: impl HasAbilityId,
    quantity: CurseCount,
) -> Result<()> {
    give_curses_with_options(game, source, quantity, GiveCurseOptions::default())
}

/// Gives curses to the Riftcaller player with the provided configuration
/// options.
pub fn give_curses_with_options(
    game: &mut GameState,
    source: impl HasAbilityId,
    quantity: CurseCount,
    options: GiveCurseOptions,
) -> Result<()> {
    state_machine::initiate(
        game,
        GiveCursesData {
            quantity,
            source: source.ability_id(),
            options,
            step: GiveCursesStep::Begin,
        },
    )
}

/// Remove *up to* `amount` curses from the Riftcaller player.
pub fn remove_curses(game: &mut GameState, amount: CurseCount) -> Result<()> {
    game.riftcaller.curse_state.base_curses =
        game.riftcaller.curse_state.base_curses.saturating_sub(amount);
    Ok(())
}

/// Returns the number of curses currently scheduled to be assigned to the
/// Riftcaller in the topmost active `give_curse` state machine.
pub fn incoming_curse_count(game: &GameState) -> Option<CurseCount> {
    game.state_machines.give_curses.last().map(|d| d.quantity)
}

/// Prevents up to `quantity` curses from being assigned to the Riftcaller in
/// the topmost active `give_curse` state machine.
pub fn prevent_curses(game: &mut GameState, quantity: CurseCount) {
    if let Some(curses) = &mut game.state_machines.give_curses.last_mut() {
        curses.quantity = curses.quantity.saturating_sub(quantity);
    }
}

/// Run the give curses state machine, if needed.
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    state_machine::run::<GiveCursesData>(game)
}

impl StateMachine for GiveCursesData {
    type Data = Self;
    type Step = GiveCursesStep;

    fn get(game: &GameState) -> &Vec<Self> {
        &game.state_machines.give_curses
    }

    fn get_mut(game: &mut GameState) -> &mut Vec<Self> {
        &mut game.state_machines.give_curses
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
        step: GiveCursesStep,
        data: GiveCursesData,
    ) -> Result<Option<GiveCursesStep>> {
        Ok(match step {
            GiveCursesStep::Begin => Some(GiveCursesStep::WillReceiveCursesEvent),
            GiveCursesStep::WillReceiveCursesEvent => {
                dispatch::invoke_event(game, WillReceiveCursesEvent(data.quantity))?;
                Some(GiveCursesStep::AddCurses)
            }
            GiveCursesStep::AddCurses => {
                if let Some(for_turn) = data.options.for_turn {
                    utils::add_matching(
                        &mut game.riftcaller.curse_state.turn_curses,
                        for_turn,
                        data.quantity,
                    );
                } else {
                    game.riftcaller.curse_state.base_curses += data.quantity;
                }
                Some(GiveCursesStep::CursesReceivedEvent)
            }
            GiveCursesStep::CursesReceivedEvent => {
                dispatch::invoke_event(game, CursesReceivedEvent(data.quantity))?;
                Some(GiveCursesStep::Finish)
            }
            GiveCursesStep::Finish => {
                game.current_history_counters(Side::Riftcaller).curses_received += data.quantity;
                None
            }
        })
    }
}
