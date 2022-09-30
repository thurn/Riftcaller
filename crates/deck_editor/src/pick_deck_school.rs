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

use core_ui::button::{Button, IconButton, IconButtonType};
use core_ui::design::FontSize;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use core_ui::text::Text;
use core_ui::{icons, panel};
use data::primitives::Side;
use panel_address::{CreateDeckState, PanelAddress};
use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition};

pub struct PickDeckSchool {
    side: Side,
}

impl PickDeckSchool {
    pub fn new(side: Side) -> Self {
        Self { side }
    }
}

impl Component for PickDeckSchool {
    fn build(self) -> Option<Node> {
        Column::new("SchoolChoice")
            .style(
                Style::new()
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center)
                    .flex_grow(1.0)
                    .width(100.pct()),
            )
            .child(
                IconButton::new(icons::BACK)
                    .action(panel::pop_to_bottom_sheet(PanelAddress::CreateDeck(
                        CreateDeckState::PickSide,
                    )))
                    .button_type(IconButtonType::SecondaryLarge)
                    .layout(
                        Layout::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Left, 20.px())
                            .position(Edge::Top, 20.px()),
                    ),
            )
            .child(Text::new(format!("Side: {:?}", self.side), FontSize::Headline))
            .child(Text::new("Pick School:", FontSize::Headline))
            .child(
                Row::new("SchoolButtons")
                    .child(
                        Button::new("Law")
                            .width_mode(WidthMode::Constrained)
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    )
                    .child(
                        Button::new("Primal")
                            .width_mode(WidthMode::Constrained)
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    )
                    .child(
                        Button::new("Shadow")
                            .width_mode(WidthMode::Constrained)
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    ),
            )
            .build()
    }
}
