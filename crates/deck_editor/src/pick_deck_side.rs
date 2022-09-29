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

use core_ui::button::{Button, IconButton};
use core_ui::design::FontSize;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use core_ui::text::Text;
use core_ui::{icons, panel};
use data::primitives::Side;
use panel_address::{CreateDeckState, PanelAddress};
use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition};

#[derive(Default)]
pub struct PickDeckSide {}

impl PickDeckSide {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for PickDeckSide {
    fn build(self) -> Option<Node> {
        Column::new("SideChoice")
            .style(
                Style::new()
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center)
                    .flex_grow(1.0)
                    .width(100.pct()),
            )
            .child(
                IconButton::new(icons::CLOSE)
                    .action(panel::close_bottom_sheet())
                    .show_frame(true)
                    .layout(
                        Layout::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Left, 20.px())
                            .position(Edge::Top, 20.px()),
                    ),
            )
            .child(Text::new("Pick Side:", FontSize::Headline))
            .child(
                Row::new("SideButtons")
                    .child(
                        Button::new("Overlord")
                            .width_mode(WidthMode::Constrained)
                            .action(panel::push_bottom_sheet(PanelAddress::CreateDeck(
                                CreateDeckState::PickSchool(Side::Overlord),
                            )))
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    )
                    .child(
                        Button::new("Champion")
                            .width_mode(WidthMode::Constrained)
                            .action(panel::push_bottom_sheet(PanelAddress::CreateDeck(
                                CreateDeckState::PickSchool(Side::Champion),
                            )))
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    ),
            )
            .build()
    }
}
