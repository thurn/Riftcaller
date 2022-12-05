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

use core_ui::button::{Button, ButtonType};
use core_ui::design::{BackgroundColor, FontSize};
use core_ui::prelude::*;
use core_ui::text::Text;
use core_ui::{panel, style};
use panel_address::PanelAddress;
use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition};

pub struct ExplorePanel {
    address: PanelAddress,
}

impl ExplorePanel {
    pub fn new(address: PanelAddress) -> Self {
        Self { address }
    }
}

impl Component for ExplorePanel {
    fn build(self) -> Option<Node> {
        Row::new("ExplorePanel")
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::All, 0.px())
                    .background_image(style::sprite("TPR/InfiniteEnvironments/meadow")),
            )
            .child(
                Column::new("Container")
                    .style(
                        Style::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Horizontal, 0.px())
                            .position(Edge::Bottom, 0.px()),
                    )
                    .child(
                        Row::new("Gradient").style(
                            Style::new()
                                .height(128.px())
                                .width(100.pct())
                                .background_image(style::sprite("Sprites/OverlayGradient")),
                        ),
                    )
                    .child(
                        Column::new("Content")
                            .style(
                                Style::new()
                                    .justify_content(FlexJustify::Center)
                                    .align_items(FlexAlign::Center)
                                    .width(100.pct())
                                    .background_color(BackgroundColor::TilePanelOverlay)
                                    .padding(Edge::All, 8.px()),
                            )
                            .child(Text::new(
                                "To the north lie the flowering fields of the Kingdom of Edennes",
                                FontSize::Headline,
                            ))
                            .child(
                                Row::new("ButtonGroup")
                                    .style(Style::new().margin(Edge::All, 8.px()))
                                    .child(
                                        Button::new("Explore \u{2022} 100 \u{f51e}")
                                            .layout(Layout::new().margin(Edge::All, 8.px())),
                                    )
                                    .child(
                                        Button::new("Close")
                                            .button_type(ButtonType::Secondary)
                                            .action(panel::close(self.address))
                                            .layout(Layout::new().margin(Edge::All, 8.px())),
                                    ),
                            ),
                    ),
            )
            .build()
    }
}
