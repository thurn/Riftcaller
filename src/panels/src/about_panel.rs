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

//! The about panel displays information about the authorship of the game

use core_ui::panel_window::PanelWindow;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use core_ui::text::Text;
use panel_address::{Panel, PanelAddress, StandardPanel};
use protos::riftcaller::{FlexAlign, FlexJustify, WhiteSpace};

use crate::main_menu_panel::{MAIN_MENU_HEIGHT, MAIN_MENU_WIDTH};

pub const TEXT: &str = "Riftcaller is open source and licensed under the Apache License, version 2.0. Source code is available at github.com/thurn/riftcaller

Music by Jay Man | OurMusicBox
Creative Commons Attribution 4.0 License
Website: www.our-music-box.com
YouTube: www.youtube.com/c/ourmusicbox

Assets are used under the terms of the Unity Asset Store License: unity3d.com/legal/as_terms";

#[derive(Debug, Default)]
pub struct AboutPanel {}

impl AboutPanel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Panel for AboutPanel {
    fn address(&self) -> PanelAddress {
        StandardPanel::About.into()
    }
}

impl Component for AboutPanel {
    fn build(self) -> Option<Node> {
        PanelWindow::new(self.address(), MAIN_MENU_WIDTH.px(), MAIN_MENU_HEIGHT.px())
            .show_close_button(true)
            .title("About")
            .content(
                Column::new("AboutContent")
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
                    ),
            )
            .build()
    }
}
