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

//! The about panel displays information about the authorship of the game

use core_ui::button::Button;
use core_ui::panel;
use core_ui::panel::Panel;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use core_ui::text::Text;
use panel_address::{PanelAddress, PanelType};
use protos::spelldawn::{FlexAlign, FlexJustify, WhiteSpace};

pub const TEXT: &str = "Welcome to the developer early access build of Spelldawn.

This is a version of the game primarily intended for contributors to the project to explore. It is a work in progress.";

#[derive(Debug, Default)]
pub struct DisclaimerPanel {}

impl DisclaimerPanel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PanelType for DisclaimerPanel {}

impl Component for DisclaimerPanel {
    fn build(self) -> Option<Node> {
        Panel::new(PanelAddress::About, 600.px(), 600.px())
            .title("Welcome!")
            .content(
                Column::new("DisclaimerContent")
                    .style(
                        Style::new()
                            .width(100.pct())
                            .align_items(FlexAlign::Stretch)
                            .justify_content(FlexJustify::Center),
                    )
                    .child(
                        ScrollView::new("TextScroll")
                            .style(Style::new().height(400.px()))
                            .child(Text::new(TEXT).white_space(WhiteSpace::Normal)),
                    )
                    .child(
                        Button::new("Understood")
                            .action(panel::set(PanelAddress::MainMenu))
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    ),
            )
            .build()
    }
}
