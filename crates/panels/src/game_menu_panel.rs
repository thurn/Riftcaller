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

//! The game menu panel provides options within a game, e.g. to return to the
//! main menu or change settings.

use core_ui::actions::InterfaceAction;
use core_ui::button::{Button, ButtonType};
use core_ui::panel_window::PanelWindow;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use core_ui::{actions, panels};
use data::game_actions::GameAction;
use panel_address::{OldDeckEditorData, Panel, PanelAddress};
use protos::spelldawn::{FlexAlign, FlexJustify};

#[derive(Debug, Default)]
pub struct GameMenuPanel {}

impl GameMenuPanel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Panel for GameMenuPanel {
    fn address(&self) -> PanelAddress {
        PanelAddress::GameMenu
    }
}

impl Component for GameMenuPanel {
    fn build(self) -> Option<Node> {
        let address = PanelAddress::GameMenu;
        PanelWindow::new(address, 512.px(), 600.px())
            .title("Menu")
            .content(
                Column::new("MeuButtons")
                    .style(
                        Style::new()
                            .width(100.pct())
                            .align_items(FlexAlign::Stretch)
                            .justify_content(FlexJustify::Center),
                    )
                    .child(menu_button("Close", panels::close(address)))
                    .child(menu_button("Resign", actions::close_and(address, GameAction::Resign)))
                    .child(menu_button(
                        "Deck Editor",
                        panels::set(PanelAddress::OldDeckEditor(OldDeckEditorData::default())),
                    )),
            )
            .build()
    }
}

fn menu_button(label: impl Into<String>, action: impl InterfaceAction + 'static) -> Button {
    Button::new(label)
        .action(action)
        .button_type(ButtonType::Primary)
        .width_mode(WidthMode::Flexible)
        .layout(Layout::new().margin(Edge::All, 16.px()))
}
