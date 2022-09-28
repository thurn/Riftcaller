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

use core_ui::button::Button;
use core_ui::design::FontSize;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use core_ui::text::Text;
use protos::spelldawn::{FlexAlign, FlexJustify, InterfacePanelAddress};

#[derive(Debug)]
#[allow(dead_code)]
pub struct CreateDeckSheet {
    address: InterfacePanelAddress,
}

impl CreateDeckSheet {
    pub fn new(address: InterfacePanelAddress) -> Self {
        Self { address }
    }
}

impl Component for CreateDeckSheet {
    fn build(self) -> Option<Node> {
        Column::new("Side Choice")
            .style(Style::new().align_items(FlexAlign::Center).justify_content(FlexJustify::Center))
            .child(Text::new("Pick Side:", FontSize::Headline))
            .child(
                Row::new("SideButtons")
                    .child(
                        Button::new("Overlord")
                            .width_mode(WidthMode::Constrained)
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    )
                    .child(
                        Button::new("Champion")
                            .width_mode(WidthMode::Constrained)
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    ),
            )
            .build()
    }
}
