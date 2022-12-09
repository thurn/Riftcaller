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

use core_ui::button::{IconButton, IconButtonType};
use core_ui::design::{Font, FontColor, FontSize};
use core_ui::icons;
use core_ui::prelude::*;
use core_ui::text::Text;
use data::player_data::PlayerData;
use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition, FontStyle, TextOverflow};

#[allow(dead_code)]
pub struct ScreenOverlay<'a> {
    player: &'a PlayerData,
}

impl<'a> ScreenOverlay<'a> {
    pub fn new(player: &'a PlayerData) -> Self {
        Self { player }
    }
}

impl<'a> Component for ScreenOverlay<'a> {
    fn build(self) -> Option<Node> {
        Row::new("ScreenOverlay")
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::Left, 1.safe_area_left())
                    .position(Edge::Right, 1.safe_area_right())
                    .position(Edge::Top, 1.safe_area_top())
                    .align_items(FlexAlign::FlexStart)
                    .justify_content(FlexJustify::SpaceBetween),
            )
            .child(
                Row::new("Left")
                    .child(
                        IconButton::new(icons::BUG)
                            .button_type(IconButtonType::NavbarBlue)
                            .layout(Layout::new().margin(Edge::All, 8.px())),
                    )
                    .child(
                        Text::new("100", FontSize::CoinCount)
                            .layout(
                                Layout::new()
                                    .margin(Edge::Left, 16.px())
                                    .margin(Edge::Top, (-12).px())
                                    .margin(Edge::Bottom, (-4).px()),
                            )
                            .font_style(FontStyle::Bold)
                            .font(Font::CoinCount)
                            .preserve_padding(true)
                            .letter_spacing((-4).px())
                            .text_overflow(TextOverflow::Clip)
                            .outline_width(3.px()),
                    )
                    .child(
                        Column::new("CoinIcon")
                            .style(Style::new().margin(Edge::Left, 8.px()))
                            .child(
                                Text::new(icons::COINS, FontSize::CoinIcon)
                                    .color(FontColor::NormalCardTitle)
                                    .layout(
                                        Layout::new()
                                            .position_type(FlexPosition::Absolute)
                                            .position(Edge::Left, (-6).px())
                                            .position(Edge::Top, 11.px()),
                                    ),
                            )
                            .child(
                                Text::new(icons::COINS, FontSize::CoinIcon).layout(
                                    Layout::new()
                                        .position_type(FlexPosition::Absolute)
                                        .position(Edge::Top, 14.px())
                                        .position(Edge::Left, (-4).px()),
                                ),
                            ),
                    ),
            )
            .child(
                Row::new("Right")
                    .child(
                        IconButton::new(icons::DECK)
                            .button_type(IconButtonType::NavbarBrown)
                            .layout(Layout::new().margin(Edge::All, 8.px())),
                    )
                    .child(
                        IconButton::new(icons::BARS)
                            .button_type(IconButtonType::NavbarBrown)
                            .layout(Layout::new().margin(Edge::All, 8.px())),
                    ),
            )
            .build()
    }
}
