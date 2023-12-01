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
use game_data::game_state::{GamePhase, GameState, PromptStack};
use game_data::prompt_data::GamePrompt;

/// Implements a simple state machine pattern. State machines have associated
/// general data as well as named individual steps. A state machine
/// consists of a function which takes a requested step, applies its associated
/// game mutations, and then returns the next step to enter.
///
/// State machines are useful because:
///  1) They are interruptible. The state machine can pause while a user-facing
///     prompt is resolved.
///  2) They repeatedly query state. A simple mutation function tends to persist
///     information in local variables, but this can cause subtle errors if game
///     mutations are triggered which update those values. The state machine
///     ensures each step is run with an up-to-date snapshot of the game state.
///
/// State machines operate on a stack (last in, first out) basis. If a second
/// instance of a state machine is initiated, that instance runs to completion
/// before the first state machine is resumed.
pub trait StateMachine: Sized {
    /// General data for all state machine states, indicating what kind of game
    /// request we are processing.
    type Data: Copy;

    /// A named step within the state machine, corresponding to some required
    /// game mutation.
    type Step: Copy;

    /// Obtain the current state machine stack from an ongoing game
    fn get(game: &GameState) -> &Vec<Self>;

    /// Mutable version of [Self::get].
    fn get_mut(game: &mut GameState) -> &mut Vec<Self>;

    /// Obtain the current state machine step from a state machine.
    fn step(&self) -> Self::Step;

    /// Mutable version of [Self::step].
    fn step_mut(&mut self) -> &mut Self::Step;

    /// Obtain a copy of the shared data for this state machine.
    fn data(&self) -> Self::Data;

    /// Run the state mutation for a given state machine step, and then return
    /// the next step to enter, or None if the state machine should be
    /// terminated.
    ///
    /// State machines may also be terminated early by dropping the state
    /// machine struct.
    fn evaluate(
        game: &mut GameState,
        step: Self::Step,
        data: Self::Data,
    ) -> Result<Option<Self::Step>>;

    /// Returns true if the current prompt in this [PromptStack] should pause
    /// the state machine.
    ///
    /// By default state machines only pause for button prompts.
    fn has_blocking_prompt(stack: &PromptStack) -> bool {
        matches!(stack.current(), Some(GamePrompt::ButtonPrompt(..)))
    }
}

/// Spawn a new instance of a state machine and start running it.
pub fn initiate<T: StateMachine>(game: &mut GameState, data: T) -> Result<()> {
    T::get_mut(game).push(data);
    run::<T>(game)
}

/// Resume an ongoing state machine, if required.
///
/// Generally, after each game action, all state machines should be polled to
/// see if they have further required updates.
pub fn run<T: StateMachine>(game: &mut GameState) -> Result<()> {
    loop {
        if T::has_blocking_prompt(&game.overlord.prompt_stack)
            || T::has_blocking_prompt(&game.champion.prompt_stack)
        {
            break;
        }

        if matches!(game.info.phase, GamePhase::GameOver { .. }) {
            break;
        }

        if let Some(current) = T::get(game).last() {
            let step = current.step();
            let data = current.data();
            if let Some(next_step) = T::evaluate(game, step, data)? {
                if let Some(current) = T::get_mut(game).last_mut() {
                    *current.step_mut() = next_step;
                }
            } else {
                T::get_mut(game).pop();
            }
        } else {
            break;
        }
    }

    Ok(())
}
