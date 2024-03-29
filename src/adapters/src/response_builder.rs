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

use std::collections::HashMap;

use core_data::game_primitives::Side;
use game_data::game_actions::DisplayPreference;
use protos::riftcaller::game_command::Command;
use protos::riftcaller::{
    CardIdentifier, GameView, ObjectPosition, PlayerName, UpdateGameViewCommand,
};

pub struct ResponseState {
    pub animate: bool,
    pub is_final_update: bool,

    /// User configuration for how this response should be rendered.
    pub display_preference: Option<DisplayPreference>,
}

/// Primary builder used to render game state.
///
/// Tracks a list of [Command]s to update the game client along with things like
/// which `Side` we are rendering for.
pub struct ResponseBuilder {
    pub user_side: Side,
    pub state: ResponseState,
    pub commands: Vec<Command>,

    /// Tracks the positions of client cards as of the most recently-seen
    /// snapshot.
    ///
    /// This is used to customize animation behavior, mostly in order to not
    /// move cards to the "display" browser when they're already in another
    /// similar card browser.
    pub last_snapshot_positions: HashMap<CardIdentifier, ObjectPosition>,
}

impl ResponseBuilder {
    pub fn new(user_side: Side, state: ResponseState) -> Self {
        Self { user_side, state, commands: vec![], last_snapshot_positions: HashMap::default() }
    }

    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn push_game_view(&mut self, game: GameView) {
        for card in &game.cards {
            if let (Some(id), Some(position)) = (card.card_id, card.card_position.clone()) {
                self.last_snapshot_positions.insert(id, position);
            }
        }

        self.commands.push(Command::UpdateGameView(UpdateGameViewCommand {
            game: Some(game),
            animate: self.state.animate,
        }));
    }

    pub fn to_player_name(&self, side: Side) -> i32 {
        if side == self.user_side {
            PlayerName::User as i32
        } else {
            PlayerName::Opponent as i32
        }
    }
}
