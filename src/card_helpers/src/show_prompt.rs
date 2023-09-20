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
use game_data::game::GameState;
use game_data::game_actions::{
    ButtonPrompt, GamePrompt, PlayCardBrowser, PromptChoice, PromptContext,
};
use game_data::game_updates::GameUpdate;
use game_data::primitives::{CardId, HasSide, Side};

/// Ads the a prompt for the `side` player containing the non-`None` actions in
/// `actions`.
pub fn with_choices(
    game: &mut GameState,
    side: Side,
    actions: Vec<Option<PromptChoice>>,
) -> Result<()> {
    game.player_mut(side).prompt_queue.push(GamePrompt::ButtonPrompt(ButtonPrompt {
        context: None,
        choices: actions.into_iter().flatten().collect(),
    }));
    Ok(())
}

pub fn play_card_browser(
    game: &mut GameState,
    side: impl HasSide,
    cards: Vec<CardId>,
    context: PromptContext,
) -> Result<()> {
    let side = side.side();
    game.record_update(|| GameUpdate::ShowPlayCardBrowser(cards.clone()));
    game.player_mut(side)
        .prompt_queue
        .push(GamePrompt::PlayCardBrowser(PlayCardBrowser { context: Some(context), cards }));
    Ok(())
}
