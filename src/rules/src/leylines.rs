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
use core_data::game_primitives::AbilityId;
use game_data::delegate_data::LeylinesReceivedEvent;
use game_data::game_state::GameState;
use game_data::state_machine_data::{GiveLeylinesData, GiveLeylinesStep};

use crate::state_machine::StateMachine;
use crate::{dispatch, state_machine};

/// Gives `quantity` leylines to the Riftcaller player. Each leyline gives the
/// Riftcaller one mana to use during each raid.
pub fn give(game: &mut GameState, source: AbilityId, quantity: u32) -> Result<()> {
    state_machine::initiate(
        game,
        GiveLeylinesData { quantity, source, step: GiveLeylinesStep::Begin },
    )
}

/// Remove up to `quantity` leylines from the Riftcaller player.
pub fn remove(game: &mut GameState, _: AbilityId, quantity: u32) -> Result<()> {
    game.riftcaller.leylines = game.riftcaller.leylines.saturating_sub(quantity);
    Ok(())
}

/// Run the state machine, if needed
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    state_machine::run::<GiveLeylinesData>(game)
}

impl StateMachine for GiveLeylinesData {
    type Step = GiveLeylinesStep;

    fn get(game: &GameState) -> &Vec<Self> {
        &game.state_machines.give_leylines
    }

    fn get_mut(game: &mut GameState) -> &mut Vec<Self> {
        &mut game.state_machines.give_leylines
    }

    fn step(&self) -> Self::Step {
        self.step
    }

    fn step_mut(&mut self) -> &mut Self::Step {
        &mut self.step
    }

    fn evaluate(
        game: &mut GameState,
        step: GiveLeylinesStep,
        data: GiveLeylinesData,
    ) -> Result<Option<GiveLeylinesStep>> {
        Ok(match step {
            GiveLeylinesStep::Begin => Some(GiveLeylinesStep::AddLeylines),
            GiveLeylinesStep::AddLeylines => {
                game.riftcaller.leylines += data.quantity;
                Some(GiveLeylinesStep::LeylinesReceivedEvent)
            }
            GiveLeylinesStep::LeylinesReceivedEvent => {
                dispatch::invoke_event(game, LeylinesReceivedEvent(&data.quantity))?;
                Some(GiveLeylinesStep::Finish)
            }
            GiveLeylinesStep::Finish => None,
        })
    }
}
