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
use game_data::special_effects::Projectile;

use crate::mana::ManaPurpose;
use crate::visual_effects::VisualEffects;
use crate::{dispatch, mana};

/// Returns true if the [Side] player currently has no active prompts.
pub fn is_empty(game: &GameState, side: Side) -> bool {
    game.player(side).prompts.stack.is_empty()
}

/// Returns the current [GamePrompt] for the [Side] player, or None if this
/// player currently has no active prompts.
pub fn current(game: &GameState, side: Side) -> Option<&GamePrompt> {
    game.player(side).prompts.stack.last().map(|e| &e.prompt)
}

/// Mutable version of [current], allows the current prompt to be mutated
/// with updated state.
pub fn current_mut(game: &mut GameState, side: Side) -> Option<&mut GamePrompt> {
    game.player_mut(side).prompts.stack.last_mut().map(|e| &mut e.prompt)
}

/// Registers an ability as wanting to show a prompt.
///
/// Abilities can request to show a prompt, and the specific value of the prompt
/// to display will be determined by invoking a delegate via [ShowPromptQuery].
/// The prompt value is computed immediately when a prompt is added to the
/// prompt stack, and then *recomputed* whenever prompt becomes the topmost
/// element in the stack again.
///
/// This allows abilities to adapt to changes in game state between the time
/// that they requested to show a prompt and the time the prompt became active.
/// If the prompt no longer makes sense in context, the ability can return None
/// to avoid showing a prompt at all.
///
/// Some common prompt cleanups are handled automatically via the prompt system.
/// For example certain [GameEffect]s are treated as "costs", and prompt choices
/// will not be shown if the player is unable to pay the associated cost.
/// Similarly a button prompt consisting only of a "continue" button will not be
/// shown.
pub fn push(game: &mut GameState, side: Side, ability_id: impl HasAbilityId) {
    push_with_data(game, side, ability_id, PromptData::None)
}

/// Equivalent function to [push] with the added ability to supply [PromptData]
/// for the ShowPrompt delegate. The data provided here will be stored and
/// included with subsequent prompt queries.
pub fn push_with_data(
    game: &mut GameState,
    side: Side,
    ability_id: impl HasAbilityId,
    data: PromptData,
) {
    let source = AbilityPromptSource { ability_id: ability_id.ability_id(), data };
    if let Some(prompt) = run_prompt_query(game, &source) {
        add_card_movement_animations(game, &prompt);
        game.player_mut(side).prompts.stack.push(PromptEntry { prompt, source: Some(source) });
    }
}

/// Add a [GamePrompt] to this player's prompt stack immediately.
///
/// This bypasses the query mechanism described above, meaning that the added
/// prompt cannot adapt to changes in game state between the time when it is
/// added to the stack and the time when it is shown.
pub fn push_immediate(game: &mut GameState, side: Side, prompt: GamePrompt) {
    add_card_movement_animations(game, &prompt);
    game.player_mut(side).prompts.stack.push(PromptEntry { prompt, source: None });
}

/// Remove and return the topmost entry in the [Side] player's prompt stack, if
/// any.
///
/// After a prompt has been handled, implementations are expected to invoke this
/// function to clear the active prompt and show the next prompt in the stack.
/// The next prompt will be regenerated via [ShowPromptQuery] as described above
/// in [push].
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

fn run_prompt_query(game: &GameState, source: &AbilityPromptSource) -> Option<GamePrompt> {
    dispatch::perform_query(game, ShowPromptQuery(source), None).and_then(|p| remove_empty(game, p))
}

/// Filters out prompts which have no effect, e.g. a button prompt with only a
/// "continue" button
fn remove_empty(game: &GameState, prompt: GamePrompt) -> Option<GamePrompt> {
    match prompt {
        GamePrompt::ButtonPrompt(mut button_prompt) => {
            button_prompt.choices.retain(|choice| valid_button_choice(game, choice));
            if button_prompt.choices.is_empty()
                || button_prompt.choices.iter().all(is_continue_choice)
            {
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

fn add_card_movement_animations(game: &mut GameState, prompt: &GamePrompt) {
    match prompt {
        GamePrompt::PlayCardBrowser(play_card) => {
            VisualEffects::new()
                .card_movement_effects(Projectile::Projectiles1(2), &play_card.cards)
                .apply(game);
        }
        GamePrompt::CardSelector(selector) => {
            VisualEffects::new()
                .card_movement_effects(Projectile::Projectiles1(2), &selector.unchosen_subjects)
                .apply(game);
        }
        _ => {}
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
