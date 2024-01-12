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
use game_data::custom_card_state::CustomCardState;
use game_data::delegate_data::Scope;
use game_data::game_state::GameState;

/// Invokes a function associated with a card in play once per turn
pub fn in_play_ability_once_per_turn(
    game: &mut GameState,
    scope: Scope,
    function: impl FnOnce(&mut GameState, Scope) -> Result<()>,
) -> Result<()> {
    let turn = game.info.turn;
    let Some(play_id) = game.card(scope).last_card_play_id else {
        return Ok(());
    };
    if !game.card(scope).custom_state.in_play_ability_triggered_for_turn(turn, play_id) {
        game.card_mut(scope)
            .custom_state
            .push(CustomCardState::InPlayAbilityTriggeredForTurn { turn, play_id });
        function(game, scope)?;
    }
    Ok(())
}

/// Invokes a function associated with an identity card once per turn
pub fn identity_once_per_turn(
    game: &mut GameState,
    scope: Scope,
    function: impl FnOnce(&mut GameState, Scope) -> Result<()>,
) -> Result<()> {
    let turn = game.info.turn;
    if !game.card(scope).custom_state.identity_triggered_for_turn(turn) {
        game.card_mut(scope).custom_state.push(CustomCardState::IdentityTriggeredForTurn { turn });
        function(game, scope)?;
    }
    Ok(())
}
