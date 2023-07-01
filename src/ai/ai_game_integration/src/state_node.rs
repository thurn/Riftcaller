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

use std::ops::{Deref, DerefMut};

use actions::legal_actions;
use ai_core::game_state_node::{GameStateNode, GameStatus};
use anyhow::Result;
use game_data::game::{GamePhase, GameState};
use game_data::game_actions::GameAction;
use game_data::primitives::Side;

/// Wrapper over [GameState] to allow trait to be implemented in this crate.
pub struct SpelldawnState(pub GameState);

impl Deref for SpelldawnState {
    type Target = GameState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SpelldawnState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl GameStateNode for SpelldawnState {
    type Action = GameAction;
    type PlayerName = Side;

    fn make_copy(&self) -> Self {
        Self(self.clone_without_updates())
    }

    fn status(&self) -> GameStatus<Side> {
        match self.info.phase {
            GamePhase::GameOver { winner } => GameStatus::Completed { winner },
            _ => {
                if actions::can_take_any_action(self, Side::Overlord) {
                    GameStatus::InProgress { current_turn: Side::Overlord }
                } else {
                    GameStatus::InProgress { current_turn: Side::Champion }
                }
            }
        }
    }

    fn legal_actions<'a>(
        &'a self,
        player: Side,
    ) -> Result<Box<dyn Iterator<Item = GameAction> + 'a>> {
        legal_actions::evaluate(self, player)
    }

    fn execute_action(&mut self, player: Side, action: GameAction) -> Result<()> {
        actions::handle_game_action(self, player, &action)
    }
}
