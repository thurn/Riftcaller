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

use ::panels::game_over_panel::GameOverPanel;
use adapters::response_builder::ResponseBuilder;
use core_ui::panels::{self, Panels};
use game_data::game::{GamePhase, GameState};
use panel_address::{GameOverData, Panel, PanelAddress};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{DisplayGameMessageCommand, GameMessageType, SetGameObjectsEnabledCommand};

pub fn check_game_over(builder: &mut ResponseBuilder, game: &GameState) {
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

        let data = GameOverData { game_id: game.id, winner };
        if let Some(panel) = (GameOverPanel { data }.build_panel()) {
            builder.push(panels::update(panel));
        }
        builder.push(Panels::open(PanelAddress::GameOver(data)).into())
    }
}
