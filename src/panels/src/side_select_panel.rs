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

//! Allows the user to select the Side they are playing as in an adventure

use core_data::game_primitives::Side;
use core_ui::button::Button;
use core_ui::panel_window::PanelWindow;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use panel_address::{Panel, PanelAddress, StandardPanel};
use protos::spelldawn::{FlexAlign, FlexJustify};
use user_action_data::UserAction;

use crate::main_menu_panel::{MAIN_MENU_HEIGHT, MAIN_MENU_WIDTH};

#[derive(Debug, Default)]
pub struct SideSelectPanel {}

impl SideSelectPanel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Panel for SideSelectPanel {
    fn address(&self) -> PanelAddress {
        StandardPanel::SideSelect.into()
    }
}

impl Component for SideSelectPanel {
    fn build(self) -> Option<Node> {
        PanelWindow::new(self.address(), MAIN_MENU_WIDTH.px(), MAIN_MENU_HEIGHT.px())
            .show_close_button(true)
            .title("New Adventure")
            .content(
                Row::new("SideSelect")
                    .style(
                        Style::new()
                            .width(100.pct())
                            .align_items(FlexAlign::Center)
                            .justify_content(FlexJustify::Center),
                    )
                    .child(side_select(Side::Overlord, "Overlord".to_string()))
                    .child(side_select(Side::Champion, "Champion".to_string())),
            )
            .build()
    }
}

fn side_select(side: Side, label: String) -> impl Component {
    Column::new(label.clone())
        .style(
            Style::new()
                .flex_grow(1.0)
                .margin(Edge::All, 16.px())
                .justify_content(FlexJustify::Center)
                .align_items(FlexAlign::Center),
        )
        .child(
            Row::new(format!("{label}Image")).style(
                Style::new()
                    .width(256.px())
                    .height(256.px())
                    .margin(Edge::All, 16.px())
                    .background_image(assets::side_badge(side)),
            ),
        )
        .child(
            Button::new(label)
                .width_mode(WidthMode::Flexible)
                .action(UserAction::NewAdventure(side)),
        )
}
