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

use adapters::response_builder::ResponseBuilder;
use core_ui::panel;
use data::game::{GamePhase, GameState};
use panel_address::{GameOverData, PanelAddress};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{DisplayGameMessageCommand, GameMessageType, SetGameObjectsEnabledCommand};

use crate::animations;

#[derive(Eq, PartialEq)]
pub enum DisplayRenderType {
    Connect,
    Update,
}

pub fn check_game_over(
    builder: &mut ResponseBuilder,
    game: &GameState,
    render_type: DisplayRenderType,
) {
    if let GamePhase::GameOver { winner } = game.data.phase {
        builder.push(Command::SetGameObjectsEnabled(SetGameObjectsEnabledCommand {
            game_objects_enabled: false,
        }));

        builder.push(Command::DisplayGameMessage(DisplayGameMessageCommand {
            message_type: if winner == builder.user_side {
                GameMessageType::Victory
            } else {
                GameMessageType::Defeat
            }
            .into(),
        }));

        builder.push(panel::open(PanelAddress::GameOver(GameOverData {
            game_id: game.id,
            winner: game.player(winner).id
        })))
    }
}
