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
use game_data::animation_tracker::GameAnimation;
use game_data::delegate_data::Scope;
use game_data::game_actions::{
    ButtonPrompt, ButtonPromptContext, GamePrompt, PlayCardBrowser, PromptChoice, PromptContext,
    UnplayedAction,
};
use game_data::game_state::GameState;
use game_data::primitives::{CardId, HasSide};

/// Adds a choice prompt for the `side` player containing the choices in
/// `choices`.
pub fn with_context_and_choices(
    game: &mut GameState,
    side: impl HasSide,
    context: ButtonPromptContext,
    choices: Vec<PromptChoice>,
) {
    game.player_mut(side.side())
        .prompt_queue
        .push(GamePrompt::ButtonPrompt(ButtonPrompt { context: Some(context), choices }))
}

/// Adds a choice prompt for the `side` player containing the choices in
/// `choices`.
pub fn with_choices(game: &mut GameState, side: impl HasSide, choices: Vec<PromptChoice>) {
    game.player_mut(side.side())
        .prompt_queue
        .push(GamePrompt::ButtonPrompt(ButtonPrompt { context: None, choices }))
}

pub fn play_card_browser(
    game: &mut GameState,
    scope: Scope,
    cards: Vec<CardId>,
    context: PromptContext,
    unplayed_action: UnplayedAction,
) -> Result<()> {
    let side = scope.side();
    game.add_animation(|| GameAnimation::ShowPlayCardBrowser(cards.clone()));
    game.player_mut(side).prompt_queue.push(GamePrompt::PlayCardBrowser(PlayCardBrowser {
        context: Some(context),
        initiated_by: scope.ability_id(),
        cards,
        unplayed_action,
    }));
    Ok(())
}

/// Show a priority window prompt if one is not already displayed. This prompt
/// allows a player to activate abilities when they otherwise could not.
pub fn priority_window(game: &mut GameState, scope: Scope) {
    if let Some(GamePrompt::PriorityPrompt) = game.player(scope.side()).prompt_queue.get(0) {
        return;
    }

    game.player_mut(scope.side()).prompt_queue.push(GamePrompt::PriorityPrompt);
}
