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
use core_data::game_primitives::{CardId, InitiatedBy, Side};
use game_data::card_state::CardPosition;
use game_data::delegate_data::{CardDestroyedEvent, WillDestroyCardEvent};
use game_data::game_state::GameState;
use game_data::state_machine_data::{DestroyPermanentData, DestroyPermanentStep};

use crate::state_machine::StateMachine;
use crate::{dispatch, mutations, state_machine};

/// Function to destroy a permanent, moving it to its owner's discard pile.
pub fn run(game: &mut GameState, target: CardId, source: InitiatedBy) -> Result<()> {
    state_machine::initiate(
        game,
        DestroyPermanentData {
            target,
            is_prevented: false,
            source,
            step: DestroyPermanentStep::Begin,
        },
    )
}

/// Run the state machine, if needed.
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    state_machine::run::<DestroyPermanentData>(game)
}

/// Prevent the [CardId] card from being destroyed if it is currently queued as
/// the target of a destroy card state machine.
pub fn prevent(_: &mut GameState, _: CardId) {}

impl StateMachine for DestroyPermanentData {
    type Data = DestroyPermanentData;
    type Step = DestroyPermanentStep;

    fn get(game: &GameState) -> &Vec<Self> {
        &game.state_machines.destroy_permanent
    }

    fn get_mut(game: &mut GameState) -> &mut Vec<Self> {
        &mut game.state_machines.destroy_permanent
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
        step: DestroyPermanentStep,
        data: DestroyPermanentData,
    ) -> Result<Option<DestroyPermanentStep>> {
        Ok(match step {
            DestroyPermanentStep::Begin => Some(DestroyPermanentStep::WillDestroyEvent),
            DestroyPermanentStep::WillDestroyEvent => {
                dispatch::invoke_event(game, WillDestroyCardEvent(data.target))?;
                Some(DestroyPermanentStep::CheckIfDestroyPrevented)
            }
            DestroyPermanentStep::CheckIfDestroyPrevented => {
                if data.is_prevented {
                    None
                } else {
                    Some(DestroyPermanentStep::Destroy)
                }
            }
            DestroyPermanentStep::Destroy => {
                mutations::move_card(
                    game,
                    data.target,
                    CardPosition::DiscardPile(data.target.side),
                )?;
                if data.target.side == Side::Riftcaller {
                    mutations::turn_face_up(game, data.target);
                }
                Some(DestroyPermanentStep::CardDestroyedEvent)
            }
            DestroyPermanentStep::CardDestroyedEvent => {
                dispatch::invoke_event(game, CardDestroyedEvent(data.target))?;
                Some(DestroyPermanentStep::Finish)
            }
            DestroyPermanentStep::Finish => None,
        })
    }
}
