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

use core_ui::actions::InterfaceAction;
use core_ui::panels::Panels;
use game_data::card_name::CardMetadata;
use game_data::card_state::CardPosition;
use game_data::game_state::GameState;
use panel_address::{ScenarioKind, StandardPanel};
use player_data::{PlayerActivity, PlayerActivityKind, PlayerState};
use protos::riftcaller::game_command::Command;
use protos::riftcaller::{KeyboardMapping, KeyboardShortcut, SetKeyboardShortcutsCommand};
use user_action_data::DebugAction;

pub fn build(player: &PlayerState, _: Option<&GameState>) -> Command {
    let activity = player.current_activity();
    let mut mapping_list = vec![];

    mapping_list.push(alt_command(
        "s",
        Panels::open(StandardPanel::ApplyScenario(ScenarioKind::Game)).wait_to_load(true),
    ));
    mapping_list.push(alt_command(
        "a",
        Panels::open(StandardPanel::ApplyScenario(ScenarioKind::Adventure)).wait_to_load(true),
    ));

    if let PlayerActivity::PlayingGame(_, user_side) = activity {
        mapping_list.push(alt_command(
            "c",
            Panels::open(StandardPanel::AddToZone {
                position: CardPosition::Hand(user_side),
                metadata: CardMetadata::default(),
                turn_face_up: false,
            })
            .wait_to_load(true),
        ));

        mapping_list.push(alt_command("m", DebugAction::AddMana(10)));
        mapping_list.push(alt_command("z", DebugAction::DebugUndo));
        mapping_list.push(alt_command(
            "d",
            Panels::open(StandardPanel::DebugPanel(
                PlayerActivityKind::PlayingGame,
                Some(user_side),
            ))
            .wait_to_load(true),
        ));
    }

    Command::SetKeyboardShortcuts(SetKeyboardShortcutsCommand { mapping_list })
}

fn alt_command(key: &'static str, action: impl InterfaceAction) -> KeyboardMapping {
    KeyboardMapping {
        shortcut: Some(KeyboardShortcut {
            key_name: key.to_string(),
            alt: true,
            ctrl: false,
            shift: false,
        }),
        action: Some(action.build()),
    }
}
