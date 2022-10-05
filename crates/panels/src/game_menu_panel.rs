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

//! The debug panel provides tools for modifying the game state during
//! development. Typically these options should not be available to production
//! users.

use core_ui::actions::InterfaceAction;
use core_ui::button::{Button, ButtonType};
use core_ui::panel;
use core_ui::panel::Panel;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use panel_address::{DeckEditorData, PanelAddress};
use protos::spelldawn::{ClientPanelAddress, FlexAlign, FlexJustify};

#[derive(Debug, Default)]
pub struct GameMenuPanel {}

impl GameMenuPanel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for GameMenuPanel {
    fn build(self) -> Option<Node> {
        let close = panel::close(panel::client(ClientPanelAddress::GameMenu));

        Panel::new(panel::client(ClientPanelAddress::GameMenu), 512.px(), 600.px())
            .title("Menu")
            .show_close_button(true)
            .content(
                Column::new("MeuButtons")
                    .style(
                        Style::new()
                            .width(100.pct())
                            .align_items(FlexAlign::Stretch)
                            .justify_content(FlexJustify::Center),
                    )
                    .child(menu_button("Resign", close.clone()))
                    .child(menu_button("Settings", close))
                    .child(menu_button(
                        "Deck Editor",
                        panel::set(PanelAddress::DeckEditor(DeckEditorData::default())),
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
