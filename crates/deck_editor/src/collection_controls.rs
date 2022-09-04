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

use core_ui::button::{Button, ButtonType, IconButton, IconButtonType};
use core_ui::design::BLUE_900;
use core_ui::icons;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use data::player_name::PlayerId;
use protos::spelldawn::FlexAlign;

#[derive(Debug)]
pub struct CollectionControls {
    player_id: PlayerId,
}

impl CollectionControls {
    pub fn new(player_id: PlayerId) -> Self {
        Self { player_id }
    }
}

impl Component for CollectionControls {
    fn build(self) -> RenderResult {
        Row::new(format!("CollectionControls for {:?}", self.player_id))
            .style(
                Style::new()
                    .background_color(BLUE_900)
                    .height(15.vh())
                    .align_items(FlexAlign::Center),
            )
            .child(
                IconButton::new(icons::CLOSE)
                    .button_type(IconButtonType::DestructiveLarge)
                    .layout(Layout::new().margin(Edge::Left, 16.px()).margin(Edge::Right, 8.px())),
            )
            .child(
                Button::new("Overlord")
                    .button_type(ButtonType::Secondary)
                    .width_mode(WidthMode::Flexible)
                    .layout(Layout::new().margin(Edge::All, 8.px())),
            )
            .child(
                Button::new("Primal")
                    .button_type(ButtonType::Secondary)
                    .width_mode(WidthMode::Flexible)
                    .layout(Layout::new().margin(Edge::All, 8.px())),
            )
            .child(
                Button::new("Filters")
                    .button_type(ButtonType::Secondary)
                    .width_mode(WidthMode::Flexible)
                    .layout(Layout::new().margin(Edge::Left, 8.px()).margin(Edge::Right, 16.px())),
            )
            .build()
    }
}
