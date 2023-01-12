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

use actions;
use anyhow::Result;
use data::game::{GameConfiguration, GameState, MulliganDecision};
use data::game_actions::{GameAction, PromptAction};
use data::player_name::{NamedPlayer, PlayerId};
use data::primitives::{GameId, Side};
use rules::{dispatch, mutations};

/// Creates a new deterministic game using the canonical decklists, deals
/// opening hands and resolves mulligans.
pub fn create() -> Result<GameState> {
    let mut game = GameState::new(
        GameId::new(0),
        PlayerId::Named(NamedPlayer::TestNoAction),
        decklists::CANONICAL_OVERLORD.clone(),
        PlayerId::Named(NamedPlayer::TestNoAction),
        decklists::CANONICAL_CHAMPION.clone(),
        GameConfiguration { deterministic: true, simulation: true },
    );

    dispatch::populate_delegate_cache(&mut game);
    mutations::deal_opening_hands(&mut game)?;
    actions::handle_game_action(
        &mut game,
        Side::Overlord,
        GameAction::PromptAction(PromptAction::MulliganDecision(MulliganDecision::Keep)),
    )?;
    actions::handle_game_action(
        &mut game,
        Side::Champion,
        GameAction::PromptAction(PromptAction::MulliganDecision(MulliganDecision::Keep)),
    )?;

    Ok(game)
}
