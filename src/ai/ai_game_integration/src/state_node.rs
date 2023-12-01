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

use std::ops::{Deref, DerefMut};

use ai_core::game_state_node::{GameStateNode, GameStatus};
use anyhow::Result;
use core_data::game_primitives::Side;
use game_data::game_actions::GameAction;
use game_data::game_state::{GamePhase, GameState};

use actions::legal_actions;
use rules::flags;

/// Wrapper over [GameState] to allow trait to be implemented in this crate.
pub struct RiftcallerState(pub GameState);

impl Deref for RiftcallerState {
    type Target = GameState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RiftcallerState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl GameStateNode for RiftcallerState {
    type Action = GameAction;
    type PlayerName = Side;

    fn make_copy(&self) -> Self {
        Self(self.clone_for_simulation())
    }

    fn status(&self) -> GameStatus<Side> {
        match self.info.phase {
            GamePhase::GameOver { winner } => GameStatus::Completed { winner },
            _ => {
                if flags::has_priority(self, Side::Overlord) {
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
