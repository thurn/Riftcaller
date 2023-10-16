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
use game_data::card_definition::{AbilityType, Cost};
use game_data::delegate_data::{AbilityActivated, ActivateAbilityEvent};
use game_data::game_actions::CardTarget;
use game_data::game_history::HistoryEvent;
use game_data::game_state::{GamePhase, GameState};
use game_data::game_updates::GameAnimation;
use game_data::primitives::AbilityId;
use game_data::state_machines::{ActivateAbilityData, ActivateAbilityStep};
use with_error::{fail, verify};

use crate::mana::ManaPurpose;
use crate::{dispatch, mana, mutations, queries, CardDefinitionExt};

/// Starts a new activate ability action
pub fn initiate(game: &mut GameState, ability_id: AbilityId, target: CardTarget) -> Result<()> {
    verify!(
        game.state_machines.activate_ability.is_none(),
        "An ability is already being resolved!"
    );
    game.state_machines.activate_ability =
        Some(ActivateAbilityData { ability_id, target, step: ActivateAbilityStep::Begin });

    run(game)
}

/// Run the activate ability state machine, if needed.
///
/// This will advance the state machine through its steps. The state machine
/// pauses if a player is presented with a prompt to respond to, and aborts if
/// the action is aborted. If no activate ability action action is currently
/// active or the state machine cannot currently advance, this function silently
/// ignores the run request.
pub fn run(game: &mut GameState) -> Result<()> {
    loop {
        if !(game.overlord.prompt_queue.is_empty() & game.champion.prompt_queue.is_empty()) {
            break;
        }

        if game.info.phase != GamePhase::Play {
            break;
        }

        if let Some(activate) = game.state_machines.activate_ability {
            let step = evaluate_step(game, activate)?;
            if let Some(updated) = &mut game.state_machines.activate_ability {
                updated.step = step;
            }
        } else {
            break;
        }
    }
    Ok(())
}

fn evaluate_step(
    game: &mut GameState,
    activate: ActivateAbilityData,
) -> Result<ActivateAbilityStep> {
    match activate.step {
        ActivateAbilityStep::Begin => Ok(ActivateAbilityStep::AddToHistory),
        ActivateAbilityStep::AddToHistory => add_to_history(game, activate),
        ActivateAbilityStep::PayActionPoints => pay_action_points(game, activate),
        ActivateAbilityStep::PayManaCost => pay_mana_cost(game, activate),
        ActivateAbilityStep::PayCustomCost => pay_custom_cost(game, activate),
        ActivateAbilityStep::Finish => finish(game, activate),
    }
}

fn add_to_history(
    game: &mut GameState,
    activate: ActivateAbilityData,
) -> Result<ActivateAbilityStep> {
    game.add_history_event(HistoryEvent::ActivateAbility(activate.ability_id, activate.target));
    Ok(ActivateAbilityStep::PayActionPoints)
}

fn pay_action_points(
    game: &mut GameState,
    activate: ActivateAbilityData,
) -> Result<ActivateAbilityStep> {
    let cost = get_cost(game, activate)?;
    mutations::spend_action_points(game, activate.ability_id.side(), cost.actions)?;
    Ok(ActivateAbilityStep::PayManaCost)
}

fn pay_mana_cost(
    game: &mut GameState,
    activate: ActivateAbilityData,
) -> Result<ActivateAbilityStep> {
    if let Some(mana) = queries::ability_mana_cost(game, activate.ability_id) {
        mana::spend(
            game,
            activate.ability_id.side(),
            ManaPurpose::ActivateAbility(activate.ability_id),
            mana,
        )?;
    }

    Ok(ActivateAbilityStep::PayCustomCost)
}

fn pay_custom_cost(
    game: &mut GameState,
    activate: ActivateAbilityData,
) -> Result<ActivateAbilityStep> {
    let cost = get_cost(game, activate)?;
    if let Some(custom_cost) = &cost.custom_cost {
        (custom_cost.pay)(game, activate.ability_id)?;
    }

    Ok(ActivateAbilityStep::Finish)
}

fn finish(game: &mut GameState, activate: ActivateAbilityData) -> Result<ActivateAbilityStep> {
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

    game.state_machines.activate_ability = None;
    Ok(ActivateAbilityStep::Finish)
}

fn get_cost(game: &GameState, activate: ActivateAbilityData) -> Result<&Cost<AbilityId>> {
    Ok(
        match &game
            .card(activate.ability_id.card_id)
            .definition()
            .ability(activate.ability_id.index)
            .ability_type
        {
            AbilityType::Activated(cost, _) => cost,
            _ => fail!("Ability is not an activated ability"),
        },
    )
}
