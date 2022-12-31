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

//! Panel rendering. A 'panel' is a discrete rectangular piece of UI which can
//! be opened or closed by the user, such as a game menu or window.

use adventure_display::adventure_panels;
use adventure_display::shop_panel::ShopPanel;
use anyhow::Result;
use data::adventure::AdventureState;
use data::player_data::PlayerData;
use deck_editor::deck_editor_panel::DeckEditorPanel;
use deck_editor::pick_deck_name::PickDeckName;
use deck_editor::pick_deck_school::PickDeckSchool;
use deck_editor::pick_deck_side::PickDeckSide;
use panel_address::{CreateDeckState, Panel, PanelAddress};
use panels::about_panel::AboutPanel;
use panels::adventure_menu::AdventureMenu;
use panels::debug_panel::DebugPanel;
use panels::disclaimer_panel::DisclaimerPanel;
use panels::game_menu_panel::GameMenuPanel;
use panels::game_over_panel::GameOverPanel;
use panels::main_menu_panel::MainMenuPanel;
use panels::set_player_name_panel::SetPlayerNamePanel;
use panels::settings_panel::SettingsPanel;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{InterfacePanel, InterfacePanelAddress, UpdatePanelsCommand};
use serde_json::de;
use with_error::WithError;

pub fn main_menu_panels() -> Vec<PanelAddress> {
    vec![
        PanelAddress::MainMenu,
        PanelAddress::Settings,
        PanelAddress::About,
        PanelAddress::Disclaimer,
    ]
}

pub fn adventure_panels(adventure: &AdventureState) -> Vec<PanelAddress> {
    adventure
        .tiles
        .iter()
        .filter_map(|(position, state)| {
            state.entity.as_ref().map(|_| PanelAddress::TileEntity(*position))
        })
        .chain(vec![PanelAddress::AdventureMenu, PanelAddress::Settings])
        .collect()
}

pub fn render_panels(
    commands: &mut Vec<Command>,
    player: &PlayerData,
    addresses: Vec<PanelAddress>,
) -> Result<()> {
    for address in addresses {
        commands.push(Command::UpdatePanels(render_panel(player, address.into())?));
    }
    Ok(())
}

pub fn render_panel(
    player: &PlayerData,
    client_address: InterfacePanelAddress,
) -> Result<UpdatePanelsCommand> {
    let server_address =
        de::from_slice(&client_address.serialized).with_error(|| "deserialization failed")?;
    let node = render_server_panel(player, server_address, client_address)?;
    Ok(UpdatePanelsCommand { panels: vec![node] })
}

fn render_server_panel(
    player: &PlayerData,
    server_address: PanelAddress,
    client_address: InterfacePanelAddress,
) -> Result<InterfacePanel> {
    Ok(match server_address {
        PanelAddress::MainMenu => MainMenuPanel::new().build_panel(),
        PanelAddress::About => AboutPanel::new().build_panel(),
        PanelAddress::Settings => SettingsPanel::new().build_panel(),
        PanelAddress::Disclaimer => DisclaimerPanel::new().build_panel(),
        PanelAddress::DebugPanel => DebugPanel::new().build_panel(),
        PanelAddress::GameMenu => GameMenuPanel::new().build_panel(),
        PanelAddress::AdventureMenu => AdventureMenu::new().build_panel(),
        PanelAddress::SetPlayerName(side) => SetPlayerNamePanel::new(side).build_panel(),
        PanelAddress::DeckEditor(data) => {
            let open_deck = if let Some(id) = data.deck { Some(player.deck(id)?) } else { None };
            DeckEditorPanel { player, open_deck, data }.build_panel()
        }
        PanelAddress::CreateDeck(state) => match state {
            CreateDeckState::PickSide => PickDeckSide::new().build_panel(),
            CreateDeckState::PickSchool(side) => PickDeckSchool::new(side).build_panel(),
            CreateDeckState::PickName(side, school) => {
                PickDeckName::new(side, school).build_panel()
            }
        },
        PanelAddress::GameOver(data) => GameOverPanel { data, player }.build_panel(),
        PanelAddress::TileEntity(position) => {
            adventure_panels::render_tile_panel(position, player, client_address)?
        }
        PanelAddress::DraftCard => render_adventure_choice(player)?,
        PanelAddress::AdventureOver => render_adventure_choice(player)?,
        PanelAddress::Shop(position) => ShopPanel::new(player, position)?.build_panel(),
    })
}

fn render_adventure_choice(player: &PlayerData) -> Result<InterfacePanel> {
    let adventure = player.adventure.as_ref().with_error(|| "Expected adventure")?;
    let rendered = adventure_display::render_adventure_choice_screen(
        adventure,
        adventure.choice_screen.as_ref().with_error(|| "Expected choice screen")?,
    )?;

    Ok(rendered.panel)
}
