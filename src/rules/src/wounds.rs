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
use core_data::game_primitives::{AbilityId, WoundCount};
use game_data::delegate_data::WoundsReceivedEvent;
use game_data::game_state::GameState;
use game_data::state_machine_data::{GiveWoundsData, GiveWoundsStep};

use crate::state_machine::StateMachine;
use crate::{dispatch, state_machine};

/// Give the Riftcaller player `quantity` wounds, which decreases their maximum
/// hand size.
pub fn give(game: &mut GameState, source: AbilityId, quantity: WoundCount) -> Result<()> {
    state_machine::initiate(game, GiveWoundsData { quantity, source, step: GiveWoundsStep::Begin })
}

/// Remove `quantity` wounds from the Riftcaller player.
pub fn remove_wounds(game: &mut GameState, quantity: WoundCount) -> Result<()> {
    game.riftcaller.wounds = game.riftcaller.wounds.saturating_sub(quantity);
    Ok(())
}

/// Run the state machine, if needed
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    state_machine::run::<GiveWoundsData>(game)
}

impl StateMachine for GiveWoundsData {
    type Step = GiveWoundsStep;

    fn get(game: &GameState) -> &Vec<Self> {
        &game.state_machines.give_wounds
    }

    fn get_mut(game: &mut GameState) -> &mut Vec<Self> {
        &mut game.state_machines.give_wounds
    }

    fn step(&self) -> Self::Step {
        self.step
    }

    fn step_mut(&mut self) -> &mut Self::Step {
        &mut self.step
    }

    fn evaluate(
        game: &mut GameState,
        step: GiveWoundsStep,
        data: GiveWoundsData,
    ) -> Result<Option<GiveWoundsStep>> {
        Ok(match step {
            GiveWoundsStep::Begin => Some(GiveWoundsStep::AddWounds),
            GiveWoundsStep::AddWounds => {
                game.riftcaller.wounds += data.quantity;
                Some(GiveWoundsStep::WoundsReceivedEvent)
            }
            GiveWoundsStep::WoundsReceivedEvent => {
                dispatch::invoke_event(game, WoundsReceivedEvent(data.quantity))?;
                Some(GiveWoundsStep::Finish)
            }
            GiveWoundsStep::Finish => None,
        })
    }
}
