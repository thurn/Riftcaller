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

use std::collections::HashSet;

use anyhow::Result;
use game_data::game_actions::{ButtonPrompt, GamePrompt, PromptChoice};
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::primitives::Side;

use crate::mana;
use crate::mana::ManaPurpose;

/// Remove user button prompts which are no longer valid as a state-based
/// action.
///
/// This function exists to inspect user prompts after every game action and
/// filter out ones which no longer make sense in context. For example, if the
/// user is presented with an option with a mana cost, but is no longer able to
/// pay that mana cost, that prompt option should be removed.
pub fn run(game: &mut GameState) -> Result<()> {
    run_for_side(game, Side::Overlord)?;
    run_for_side(game, Side::Champion)?;
    Ok(())
}

fn run_for_side(game: &mut GameState, side: Side) -> Result<()> {
    let Some(GamePrompt::ButtonPrompt(button_prompt)) = game.player(side).prompt_stack.current()
    else {
        return Ok(());
    };

    let indices_to_remove = button_prompt
        .choices
        .iter()
        .enumerate()
        .filter(|(_, choice)| !can_pay_prompt_cost(game, choice))
        .map(|(i, _)| i)
        .collect::<HashSet<_>>();
    if !indices_to_remove.is_empty() {
        let new = GamePrompt::ButtonPrompt(remove_choice_indices(button_prompt, indices_to_remove));
        game.player_mut(side).prompt_stack.pop();
        game.player_mut(side).prompt_stack.push(new);
    }

    Ok(())
}

fn remove_choice_indices(prompt: &ButtonPrompt, indices: HashSet<usize>) -> ButtonPrompt {
    // There is probably a more efficient way to do this with in-place mutation
    ButtonPrompt {
        context: prompt.context.clone(),
        choices: prompt
            .choices
            .iter()
            .cloned()
            .enumerate()
            .filter(|(i, _)| !indices.contains(i))
            .map(|(_, choice)| choice)
            .collect(),
    }
}

fn can_pay_prompt_cost(game: &GameState, choice: &PromptChoice) -> bool {
    choice.effects.iter().all(|e| can_pay_effect_cost(game, e))
}

fn can_pay_effect_cost(game: &GameState, effect: &GameEffect) -> bool {
    match effect {
        GameEffect::ManaCost(side, cost) => {
            mana::get(game, *side, ManaPurpose::PayForTriggeredAbility) >= *cost
        }
        GameEffect::ActionCost(side, cost) => game.player(*side).actions >= *cost,
        GameEffect::TakeDamageCost(_, amount) => {
            game.hand(Side::Champion).count() >= *amount as usize
        }
        _ => true,
    }
}
