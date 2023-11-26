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
use game_data::game_actions::GamePrompt;
use game_data::game_state::{GamePhase, GameState, PromptStack};

pub trait StateMachine: Sized {
    type Data: Copy;
    type Step: Copy;

    fn get(game: &GameState) -> &Vec<Self>;

    fn get_mut(game: &mut GameState) -> &mut Vec<Self>;

    fn step(&self) -> Self::Step;

    fn step_mut(&mut self) -> &mut Self::Step;

    fn data(&self) -> Self::Data;

    fn evaluate(
        game: &mut GameState,
        step: Self::Step,
        data: Self::Data,
    ) -> Result<Option<Self::Step>>;

    fn has_blocking_prompt(stack: &PromptStack) -> bool {
        matches!(stack.current(), Some(GamePrompt::ButtonPrompt(..)))
    }
}

pub fn initiate<T: StateMachine>(game: &mut GameState, data: T) -> Result<()> {
    T::get_mut(game).push(data);
    run::<T>(game)
}

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
