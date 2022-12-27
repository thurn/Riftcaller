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
use core_ui::panels;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use panel_address::PanelAddress;
use protos::spelldawn::{FlexAlign, FlexJustify};

/// Displays a panel menu consisting of a series of buttons
pub struct ButtonMenu {
    address: PanelAddress,
    title: String,
    children: Vec<Box<dyn ComponentObject>>,
    show_close_button: bool,
}

impl ButtonMenu {
    pub fn new(address: PanelAddress) -> Self {
        Self { address, title: "Menu".to_string(), children: vec![], show_close_button: true }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn button(
        mut self,
        label: impl Into<String>,
        action: impl InterfaceAction + 'static,
    ) -> Self {
        self.children.push(Box::new(menu_button(label, action)));
        self
    }

    pub fn show_close_button(mut self, show_close_button: bool) -> Self {
        self.show_close_button = show_close_button;
        self
    }
}

impl Component for ButtonMenu {
    fn build(mut self) -> Option<Node> {
        if self.show_close_button {
            self.children.push(Box::new(menu_button("Close", panels::close(self.address))));
        }

        PanelWindow::new(self.address, 512.px(), 600.px())
            .title(self.title)
            .content(
                Column::new("MeuButtons")
                    .style(
                        Style::new()
                            .width(100.pct())
                            .align_items(FlexAlign::Stretch)
                            .justify_content(FlexJustify::Center),
                    )
                    .children_boxed(self.children),
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
