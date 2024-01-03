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
use core_data::game_primitives::InitiatedBy;
use dispatcher::dispatch;
use game_data::delegate_data::{
    AbilityWillEndRaidEvent, RaidEndEvent, RaidFailureEvent, RaidOutcome, RaidSuccessEvent,
};
use game_data::game_state::GameState;
use game_data::history_data::HistoryEvent;
use game_data::state_machine_data::{EndRaidData, EndRaidStep};

use crate::state_machine::StateMachine;
use crate::{flags, state_machine};

/// Ends the current raid.
///
/// Has no effect if no raid is currently active.
pub fn run(game: &mut GameState, source: InitiatedBy, outcome: RaidOutcome) -> Result<()> {
    if !flags::raid_active(game) {
        return Ok(());
    }

    state_machine::initiate(
        game,
        EndRaidData { outcome, source, prevented: false, step: EndRaidStep::Begin },
    )
}

/// Run the state machine, if needed.
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    state_machine::run::<EndRaidData>(game)
}

/// Prevent the current raid from being ended.
///
/// Has no effect if no 'end raid' event is currently active.
pub fn prevent(game: &mut GameState) {
    if let Some(end_raid) = game.state_machines.end_raid.last_mut() {
        end_raid.prevented = true;
    }
}

impl StateMachine for EndRaidData {
    type Step = EndRaidStep;

    fn get(game: &GameState) -> &Vec<Self> {
        &game.state_machines.end_raid
    }

    fn get_mut(game: &mut GameState) -> &mut Vec<Self> {
        &mut game.state_machines.end_raid
    }

    fn step(&self) -> Self::Step {
        self.step
    }

    fn step_mut(&mut self) -> &mut Self::Step {
        &mut self.step
    }

    fn evaluate(
        game: &mut GameState,
        step: EndRaidStep,
        data: EndRaidData,
    ) -> Result<Option<EndRaidStep>> {
        Ok(match step {
            EndRaidStep::Begin => Some(EndRaidStep::FireAbilityWillEndRaid),
            EndRaidStep::FireAbilityWillEndRaid => {
                let info = game.raid()?.info();
                if let InitiatedBy::Ability(ability_id) = data.source {
                    dispatch::invoke_event(game, AbilityWillEndRaidEvent(&info.event(ability_id)))?;
                }
                Some(EndRaidStep::CheckIfPrevented)
            }
            EndRaidStep::CheckIfPrevented => {
                if data.prevented {
                    None
                } else {
                    Some(EndRaidStep::FireOutcomeEvents)
                }
            }
            EndRaidStep::FireOutcomeEvents => {
                let info = game.raid()?.info();
                let event = info.event(());
                match data.outcome {
                    RaidOutcome::Success => {
                        dispatch::invoke_event(game, RaidSuccessEvent(&event))?;
                    }
                    RaidOutcome::Failure => {
                        dispatch::invoke_event(game, RaidFailureEvent(&event))?;
                    }
                }
                Some(EndRaidStep::AddToHistory)
            }
            EndRaidStep::AddToHistory => {
                let info = game.raid()?.info();
                let event = info.event(());
                match data.outcome {
                    RaidOutcome::Success => {
                        game.add_history_event(HistoryEvent::RaidSuccess(event));
                    }
                    RaidOutcome::Failure => {
                        game.add_history_event(HistoryEvent::RaidFailure(event));
                    }
                }
                Some(EndRaidStep::FireEndRaidEvent)
            }
            EndRaidStep::FireEndRaidEvent => {
                let info = game.raid()?.info();
                dispatch::invoke_event(game, RaidEndEvent(&info.event(data.outcome)))?;
                Some(EndRaidStep::EndRaid)
            }
            EndRaidStep::EndRaid => {
                game.raid = None;
                Some(EndRaidStep::Finish)
            }
            EndRaidStep::Finish => None,
        })
    }
}
