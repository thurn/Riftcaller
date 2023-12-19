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

use core_data::game_primitives::Side;
use panel_address::{PlayerPanel, StandardPanel};
use player_data::{PlayerActivityKind, PlayerState};

/// Enumerates all standard panel addresses
pub fn standard_panels() -> Vec<StandardPanel> {
    vec![
        StandardPanel::MainMenu,
        StandardPanel::About,
        StandardPanel::Settings,
        StandardPanel::SideSelect,
        StandardPanel::Disclaimer,
        StandardPanel::DebugPanel(PlayerActivityKind::None, None),
        StandardPanel::DebugPanel(PlayerActivityKind::Adventure, None),
        StandardPanel::DebugPanel(PlayerActivityKind::PlayingGame, Some(Side::Covenant)),
        StandardPanel::DebugPanel(PlayerActivityKind::PlayingGame, Some(Side::Riftcaller)),
        StandardPanel::GameMenu,
        StandardPanel::AdventureMenu,
        StandardPanel::SetPlayerName(Side::Riftcaller),
        StandardPanel::SetPlayerName(Side::Covenant),
        StandardPanel::DeckEditorLoading,
    ]
}

/// Enumerates all player panel addresses
pub fn player_panels(player: &PlayerState) -> Vec<PlayerPanel> {
    let mut panels =
        vec![PlayerPanel::AdventureOver, PlayerPanel::BattleVictory, PlayerPanel::BattleDefeat];
    if let Some(adventure) = &player.adventure {
        for i in 0..adventure.screens.count() {
            panels.push(PlayerPanel::AdventureScreen(i));
        }
        panels.push(PlayerPanel::DeckEditor(None));
        panels
    } else {
        panels
    }
}
