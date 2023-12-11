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

use core_data::game_primitives::{HasAbilityId, Side};
use game_data::delegate_data::ShowPromptQuery;
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::prompt_data::{
    AbilityPromptSource, GamePrompt, PromptChoice, PromptData, PromptEntry,
};

use crate::mana::ManaPurpose;
use crate::{dispatch, mana};

pub fn is_empty(game: &GameState, side: Side) -> bool {
    game.player(side).prompts.stack.is_empty()
}

pub fn push(game: &mut GameState, side: Side, ability_id: impl HasAbilityId) {
    push_with_data(game, side, ability_id, PromptData::None)
}

pub fn push_with_data(
    game: &mut GameState,
    side: Side,
    ability_id: impl HasAbilityId,
    data: PromptData,
) {
    let source = AbilityPromptSource { ability_id: ability_id.ability_id(), data };
    if let Some(prompt) = run_prompt_query(game, &source) {
        game.player_mut(side).prompts.stack.push(PromptEntry { prompt, source: Some(source) });
    }
}

pub fn push_immediate(game: &mut GameState, side: Side, prompt: GamePrompt) {
    game.player_mut(side).prompts.stack.push(PromptEntry { prompt, source: None });
}

pub fn pop(game: &mut GameState, side: Side) -> Option<GamePrompt> {
    let previous = game.player_mut(side).prompts.stack.pop();
    while let Some(current) = game.player_mut(side).prompts.stack.pop() {
        if let Some(source) = current.source {
            if let Some(prompt) = run_prompt_query(game, &source) {
                // Updated prompt returned
                game.player_mut(side)
                    .prompts
                    .stack
                    .push(PromptEntry { prompt, source: Some(source) });
                break;
            } else {
                // Current prompt removed when recomputed, continue to the next
                // entry in the stack.
            }
        } else {
            // Current prompt has no source, do not recompute
            break;
        }
    }
    previous.map(|p| p.prompt)
}

pub fn current_mut(game: &mut GameState, side: Side) -> Option<&mut GamePrompt> {
    game.player_mut(side).prompts.stack.last_mut().map(|e| &mut e.prompt)
}

pub fn current(game: &GameState, side: Side) -> Option<&GamePrompt> {
    game.player(side).prompts.stack.last().map(|e| &e.prompt)
}

fn run_prompt_query(game: &GameState, source: &AbilityPromptSource) -> Option<GamePrompt> {
    dispatch::perform_query(game, ShowPromptQuery(source), None).and_then(|p| remove_empty(game, p))
}

/// Filters out prompts which have no effect, e.g. a button prompt with only a
/// "continue" button
fn remove_empty(game: &GameState, prompt: GamePrompt) -> Option<GamePrompt> {
    match prompt {
        GamePrompt::ButtonPrompt(mut button_prompt) => {
            button_prompt.choices.retain(|choice| valid_button_choice(game, choice));
            if button_prompt.choices.iter().all(is_continue_choice) {
                None
            } else {
                Some(GamePrompt::ButtonPrompt(button_prompt))
            }
        }
        GamePrompt::CardSelector(card_selector) => {
            if card_selector.chosen_subjects.is_empty()
                && card_selector.unchosen_subjects.is_empty()
            {
                None
            } else {
                Some(GamePrompt::CardSelector(card_selector))
            }
        }
        GamePrompt::PlayCardBrowser(play_card_browser) => {
            if play_card_browser.cards.is_empty() {
                None
            } else {
                Some(GamePrompt::PlayCardBrowser(play_card_browser))
            }
        }
        GamePrompt::PriorityPrompt => Some(GamePrompt::PriorityPrompt),
        GamePrompt::RoomSelector(room_selector) => {
            if room_selector.valid_rooms.is_empty() {
                None
            } else {
                Some(GamePrompt::RoomSelector(room_selector))
            }
        }
    }
}

fn valid_button_choice(game: &GameState, choice: &PromptChoice) -> bool {
    choice.effects.iter().all(|effect| match effect {
        GameEffect::ManaCost(side, cost, _) => {
            mana::get(game, *side, ManaPurpose::PayForTriggeredAbility) >= *cost
        }
        GameEffect::ActionCost(side, cost) => game.player(*side).actions >= *cost,
        GameEffect::TakeDamageCost(_, amount) => {
            game.hand(Side::Riftcaller).count() >= *amount as usize
        }
        _ => true,
    })
}

fn is_continue_choice(choice: &PromptChoice) -> bool {
    choice.effects.iter().all(|effect| effect == &GameEffect::Continue)
}
