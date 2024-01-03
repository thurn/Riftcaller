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
use dispatcher::dispatch;
use game_data::card_state::CardPosition;
use game_data::delegate_data::{CardsDestroyedEvent, WillDestroyCardsEvent};
use game_data::game_state::GameState;
use game_data::state_machine_data::{DestroyPermanentStep, DestroyPermanentsData};

use crate::state_machine::StateMachine;
use crate::{mutations, state_machine};

/// Function to destroy one or more permanents, moving them to their owner's
/// discard pile.
pub fn run(game: &mut GameState, targets: Vec<CardId>, source: InitiatedBy) -> Result<()> {
    state_machine::initiate(
        game,
        DestroyPermanentsData { targets, source, step: DestroyPermanentStep::Begin },
    )
}

/// Run the state machine, if needed.
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    state_machine::run::<DestroyPermanentsData>(game)
}

/// Returns all [CardId]s which are currently targeted for destruction as part
/// of a destroy action.
pub fn all_targets(game: &GameState) -> Vec<CardId> {
    game.state_machines
        .destroy_permanent
        .iter()
        .flat_map(|state_machine| &state_machine.targets)
        .copied()
        .collect()
}

/// Prevent the [CardId] card from being destroyed if it is currently queued as
/// the target of a destroy card state machine.
pub fn prevent(game: &mut GameState, card_id: CardId) {
    for state_machine in &mut game.state_machines.destroy_permanent.iter_mut().rev() {
        if state_machine.targets.contains(&card_id) {
            state_machine.targets.retain(|c| *c != card_id);
            break;
        }
    }
}

impl StateMachine for DestroyPermanentsData {
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

    fn evaluate(
        game: &mut GameState,
        step: DestroyPermanentStep,
        data: DestroyPermanentsData,
    ) -> Result<Option<DestroyPermanentStep>> {
        Ok(match step {
            DestroyPermanentStep::Begin => Some(DestroyPermanentStep::WillDestroyEvent),
            DestroyPermanentStep::WillDestroyEvent => {
                dispatch::invoke_event(game, WillDestroyCardsEvent(&data.targets))?;
                Some(DestroyPermanentStep::CheckIfDestroyPrevented)
            }
            DestroyPermanentStep::CheckIfDestroyPrevented => {
                if data.targets.is_empty() {
                    None
                } else {
                    Some(DestroyPermanentStep::Destroy)
                }
            }
            DestroyPermanentStep::Destroy => {
                for card_id in &data.targets {
                    mutations::move_card(game, *card_id, CardPosition::DiscardPile(card_id.side))?;
                    if card_id.side == Side::Riftcaller {
                        mutations::turn_face_up(game, *card_id)
                    }
                }

                Some(DestroyPermanentStep::CardsDestroyedEvent)
            }
            DestroyPermanentStep::CardsDestroyedEvent => {
                dispatch::invoke_event(game, CardsDestroyedEvent(&data.targets))?;
                Some(DestroyPermanentStep::Finish)
            }
            DestroyPermanentStep::Finish => None,
        })
    }
}
