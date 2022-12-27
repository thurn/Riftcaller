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

//! The settings panel allows configuration of game options

use core_ui::button::Button;
use core_ui::panel_window::PanelWindow;
use core_ui::panels;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use core_ui::slider::Slider;
use panel_address::{Panel, PanelAddress};
use protos::spelldawn::{FlexAlign, FlexJustify};

#[derive(Debug, Default)]
pub struct SettingsPanel {}

impl SettingsPanel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Panel for SettingsPanel {
    fn address(&self) -> PanelAddress {
        PanelAddress::Settings
    }
}

impl Component for SettingsPanel {
    fn build(self) -> Option<Node> {
        PanelWindow::new(PanelAddress::Settings, 600.px(), 600.px())
            .title("Settings")
            .content(
                Column::new("SettingsContent")
                    .style(
                        Style::new()
                            .width(100.pct())
                            .align_items(FlexAlign::Stretch)
                            .justify_content(FlexJustify::Center),
                    )
                    .child(
                        ScrollView::new("TextScroll").style(Style::new().height(400.px())).child(
                            Slider::new()
                                .label("Music Volume:")
                                .preference_key("MusicVolume")
                                .low_value(0.0)
                                .high_value(1.0),
                        ),
                    )
                    .child(
                        Button::new("Back")
                            .action(panels::close(PanelAddress::Settings))
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    ),
            )
            .build()
    }
}
