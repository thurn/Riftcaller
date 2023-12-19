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

//! Panel rendering. A 'panel' is a discrete rectangular piece of UI which can
//! be opened or closed by the user, such as a game menu or window.

use adventure_display::adventure_over_panel::AdventureOverPanel;
use adventure_display::adventure_panels;
use anyhow::Result;
use deck_editor::deck_editor_panel::DeckEditorPanel;
use panel_address::{Panel, PlayerPanel, StandardPanel};
use panels::about_panel::AboutPanel;
use panels::add_to_zone_panel::AddToZonePanel;
use panels::adventure_menu::AdventureMenu;
use panels::apply_scenario_panel::ApplyScenarioPanel;
use panels::battle_defeat_panel::BattleDefeatPanel;
use panels::battle_victory_panel::BattleVictoryPanel;
use panels::debug_create_card_panel::DebugCreateCardPanel;
use panels::debug_panel::DebugPanel;
use panels::disclaimer_panel::DisclaimerPanel;
use panels::game_menu_panel::GameMenuPanel;
use panels::loading_panel::LoadingPanel;
use panels::main_menu_panel::MainMenuPanel;
use panels::set_player_name_panel::SetPlayerNamePanel;
use panels::settings_panel::SettingsPanel;
use panels::side_select_panel::SideSelectPanel;
use player_data::PlayerState;
use protos::riftcaller::InterfacePanel;

pub mod all_panels;

pub fn render_standard_panel(panel: StandardPanel) -> Result<Option<InterfacePanel>> {
    Ok(match panel {
        StandardPanel::MainMenu => MainMenuPanel::new().build_panel(),
        StandardPanel::About => AboutPanel::new().build_panel(),
        StandardPanel::Settings => SettingsPanel::new().build_panel(),
        StandardPanel::SideSelect => SideSelectPanel::new().build_panel(),
        StandardPanel::Disclaimer => DisclaimerPanel::new().build_panel(),
        StandardPanel::DebugPanel(activity, side) => DebugPanel::new(activity, side).build_panel(),
        StandardPanel::GameMenu => GameMenuPanel::new().build_panel(),
        StandardPanel::AdventureMenu => AdventureMenu::new().build_panel(),
        StandardPanel::DeckEditorLoading => LoadingPanel::new(
            panel.into(),
            "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Library/SceneryLibrary_inside_1",
        )
        .build_panel(),
        StandardPanel::SetPlayerName(side) => SetPlayerNamePanel::new(side).build_panel(),
        StandardPanel::DebugCreateCard(side, metadata) => {
            DebugCreateCardPanel::new(side, metadata).build_panel()
        }
        StandardPanel::AddToZone { position, metadata, turn_face_up } => {
            AddToZonePanel::new("", position, metadata, turn_face_up).build_panel()
        }
        StandardPanel::ApplyScenario(kind) => ApplyScenarioPanel::new(kind).build_panel(),
    })
}

pub fn render_player_panel(
    player: &PlayerState,
    address: PlayerPanel,
) -> Result<Option<InterfacePanel>> {
    Ok(match address {
        PlayerPanel::DeckEditor(action) => DeckEditorPanel { player, action }.build_panel(),
        PlayerPanel::BattleVictory => BattleVictoryPanel::new(player).build_panel(),
        PlayerPanel::BattleDefeat => BattleDefeatPanel {}.build_panel(),
        PlayerPanel::AdventureScreen(index) => adventure_panels::tile_entity_panel(player, index)?,
        PlayerPanel::AdventureOver => AdventureOverPanel::new().build_panel(),
    })
}
