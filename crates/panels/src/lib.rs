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

pub mod about_panel;
pub mod adventure_menu;
pub mod adventure_over_panel;
pub mod button_menu;
pub mod debug_panel;
pub mod disclaimer_panel;
pub mod game_menu_panel;
pub mod game_over_panel;
pub mod main_menu_panel;
pub mod set_player_name_panel;
pub mod settings_panel;

use adventure_display::adventure_panels;
use adventure_display::draft_panel::DraftPanel;
use anyhow::Result;
use core_ui::component::Component;
use data::adventure::AdventureState;
use data::player_data::PlayerData;
use debug_panel::DebugPanel;
use deck_editor::deck_editor_panel::DeckEditorPanel;
use deck_editor::pick_deck_name::PickDeckName;
use deck_editor::pick_deck_school::PickDeckSchool;
use deck_editor::pick_deck_side::PickDeckSide;
use panel_address::{CreateDeckState, PanelAddress};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::interface_panel_address::AddressType;
use protos::spelldawn::{
    ClientPanelAddress, InterfacePanel, InterfacePanelAddress, Node, UpdatePanelsCommand,
};
use serde_json::de;
use with_error::WithError;

use crate::about_panel::AboutPanel;
use crate::adventure_menu::AdventureMenu;
use crate::adventure_over_panel::AdventureOverPanel;
use crate::disclaimer_panel::DisclaimerPanel;
use crate::game_menu_panel::GameMenuPanel;
use crate::game_over_panel::GameOverPanel;
use crate::main_menu_panel::MainMenuPanel;
use crate::set_player_name_panel::SetPlayerNamePanel;
use crate::settings_panel::SettingsPanel;

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
        .filter_map(|(position, state)| state.entity.map(|_| PanelAddress::TileEntity(*position)))
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
    address: InterfacePanelAddress,
) -> Result<UpdatePanelsCommand> {
    let node = match address.address_type.as_ref().with_error(|| "missing address_type")? {
        AddressType::Serialized(payload) => {
            let address = de::from_slice(payload).with_error(|| "deserialization failed")?;
            render_server_panel(player, address)?
        }
        AddressType::ClientPanel(client_panel) => render_client_panel(
            ClientPanelAddress::from_i32(*client_panel).with_error(|| "invalid known panel")?,
        ),
    };

    Ok(UpdatePanelsCommand { panels: vec![InterfacePanel { address: Some(address), node }] })
}

fn render_server_panel(player: &PlayerData, address: PanelAddress) -> Result<Option<Node>> {
    Ok(match address {
        PanelAddress::MainMenu => MainMenuPanel::new().build(),
        PanelAddress::About => AboutPanel::new().build(),
        PanelAddress::Settings => SettingsPanel::new().build(),
        PanelAddress::Disclaimer => DisclaimerPanel::new().build(),
        PanelAddress::AdventureMenu => AdventureMenu::new().build(),
        PanelAddress::AdventureOver => AdventureOverPanel::new().build(),
        PanelAddress::SetPlayerName(side) => SetPlayerNamePanel::new(side).build(),
        PanelAddress::DeckEditor(data) => {
            let open_deck = if let Some(id) = data.deck { Some(player.deck(id)?) } else { None };
            DeckEditorPanel {
                player,
                open_deck,
                filters: data.collection_filters,
                show_edit_options: data.show_edit_options,
            }
            .build()
        }
        PanelAddress::CreateDeck(state) => match state {
            CreateDeckState::PickSide => PickDeckSide::new().build(),
            CreateDeckState::PickSchool(side) => PickDeckSchool::new(side).build(),
            CreateDeckState::PickName(side, school) => PickDeckName::new(side, school).build(),
        },
        PanelAddress::GameOver(_) => GameOverPanel { address, player }.build(),
        PanelAddress::TileEntity(position) => adventure_panels::render(position, player)?,
        PanelAddress::Draft => DraftPanel { address }.build(),
    })
}

fn render_client_panel(address: ClientPanelAddress) -> Option<Node> {
    match address {
        ClientPanelAddress::Unspecified => None,
        ClientPanelAddress::DebugPanel => DebugPanel::new().build(),
        ClientPanelAddress::GameMenu => GameMenuPanel::new().build(),
    }
}
