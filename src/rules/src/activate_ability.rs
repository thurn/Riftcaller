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
use core_data::game_primitives::{AbilityId, InitiatedBy};
use game_data::animation_tracker::GameAnimation;
use game_data::card_definition::{AbilityType, Cost};
use game_data::delegate_data::{AbilityActivated, ActivateAbilityEvent};
use game_data::game_actions::CardTarget;
use game_data::game_state::GameState;
use game_data::history_data::{AbilityActivation, AbilityActivationType, HistoryEvent};
use game_data::state_machine_data::{ActivateAbilityData, ActivateAbilityStep};
use with_error::fail;

use crate::mana::ManaPurpose;
use crate::state_machine::StateMachine;
use crate::{dispatch, mana, mutations, queries, state_machine, CardDefinitionExt};

/// Starts a new activate ability action
pub fn initiate(game: &mut GameState, ability_id: AbilityId, target: CardTarget) -> Result<()> {
    state_machine::initiate(
        game,
        ActivateAbilityData { ability_id, target, step: ActivateAbilityStep::Begin },
    )
}

/// Run the activate ability state machine, if needed.
///
/// This will advance the state machine through its steps. The state machine
/// pauses if a player is presented with a prompt to respond to, and aborts if
/// the action is aborted. If no activate ability action action is currently
/// active or the state machine cannot currently advance, this function silently
/// ignores the run request.
pub fn run(game: &mut GameState) -> Result<()> {
    state_machine::run::<ActivateAbilityData>(game)
}

/// Returns true if the provided [AbilityId] is currently being resolved by the
/// current `activate_ability` state machine.
pub fn is_current_ability(game: &GameState, ability_id: AbilityId) -> bool {
    game.state_machines.activate_ability.last().map(|d| d.ability_id) == Some(ability_id)
}

impl StateMachine for ActivateAbilityData {
    type Data = ActivateAbilityData;
    type Step = ActivateAbilityStep;

    fn get(game: &GameState) -> &Vec<Self> {
        &game.state_machines.activate_ability
    }

    fn get_mut(game: &mut GameState) -> &mut Vec<Self> {
        &mut game.state_machines.activate_ability
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
        step: ActivateAbilityStep,
        data: ActivateAbilityData,
    ) -> Result<Option<ActivateAbilityStep>> {
        match step {
            ActivateAbilityStep::Begin => Ok(Some(ActivateAbilityStep::PayActionPoints)),
            ActivateAbilityStep::PayActionPoints => pay_action_points(game, data),
            ActivateAbilityStep::PayManaCost => pay_mana_cost(game, data),
            ActivateAbilityStep::PayCustomCost => pay_custom_cost(game, data),
            ActivateAbilityStep::Finish => finish(game, data),
        }
    }
}

fn pay_action_points(
    game: &mut GameState,
    activate: ActivateAbilityData,
) -> Result<Option<ActivateAbilityStep>> {
    let actions = get_cost(game, activate)?.actions;

    let activation = AbilityActivation {
        ability_id: activate.ability_id,
        target: activate.target,
        activation_type: if actions > 0 {
            AbilityActivationType::GameAction
        } else {
            AbilityActivationType::FreeAction
        },
        current_raid: game.raid.as_ref().map(|r| r.raid_id),
        current_minion_encounter: game.raid.as_ref().and_then(|r| r.minion_encounter_id),
        current_room_access: game.raid.as_ref().and_then(|r| r.room_access_id),
    };

    game.add_history_event(HistoryEvent::ActivateAbility(activation));

    mutations::spend_action_points(game, activate.ability_id.side(), actions)?;
    Ok(Some(ActivateAbilityStep::PayManaCost))
}

fn pay_mana_cost(
    game: &mut GameState,
    activate: ActivateAbilityData,
) -> Result<Option<ActivateAbilityStep>> {
    if let Some(mana) = queries::ability_mana_cost(game, activate.ability_id) {
        mana::spend(
            game,
            activate.ability_id.side(),
            InitiatedBy::Ability(activate.ability_id),
            ManaPurpose::ActivateAbility(activate.ability_id),
            mana,
        )?;
    }

    Ok(Some(ActivateAbilityStep::PayCustomCost))
}

fn pay_custom_cost(
    game: &mut GameState,
    activate: ActivateAbilityData,
) -> Result<Option<ActivateAbilityStep>> {
    let cost = get_cost(game, activate)?;
    if let Some(custom_cost) = &cost.custom_cost {
        (custom_cost.pay)(game, activate.ability_id)?;
    }

    Ok(Some(ActivateAbilityStep::Finish))
}

fn finish(
    game: &mut GameState,
    activate: ActivateAbilityData,
) -> Result<Option<ActivateAbilityStep>> {
    game.add_animation(|| {
        GameAnimation::AbilityActivated(activate.ability_id.side(), activate.ability_id)
    });

    dispatch::invoke_event(
        game,
        ActivateAbilityEvent(AbilityActivated {
            ability_id: activate.ability_id,
            target: activate.target,
        }),
    )?;

    Ok(None)
}

fn get_cost(game: &GameState, activate: ActivateAbilityData) -> Result<&Cost<AbilityId>> {
    Ok(
        match &game
            .card(activate.ability_id.card_id)
            .definition()
            .ability(activate.ability_id.index)
            .ability_type
        {
            AbilityType::Activated { cost, .. } => cost,
            _ => fail!("Ability is not an activated ability"),
        },
    )
}
