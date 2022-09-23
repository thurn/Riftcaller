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

use protos::spelldawn::{FlexPosition, InterfacePanelAddress};

use crate::component::EmptyComponent;
use crate::design::BackgroundColor;
use crate::prelude::*;
use crate::style::Corner;

#[allow(dead_code)]
pub struct BottomSheet {
    address: InterfacePanelAddress,
    content: Box<dyn Component>,
}

impl BottomSheet {
    pub fn new(address: impl Into<InterfacePanelAddress>) -> Self {
        Self { address: address.into(), content: Box::new(EmptyComponent) }
    }

    pub fn content(mut self, content: impl Component + 'static) -> Self {
        self.content = Box::new(content);
        self
    }
}

impl Component for BottomSheet {
    fn build(self) -> Option<Node> {
        Row::new("BottomSheetOverlay")
            .style(
                Style::new()
                    .background_color(BackgroundColor::BottomSheetOverlay)
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::All, 0.px()),
            )
            .child(
                Column::new("BottomSheet").style(
                    Style::new()
                        .background_color(BackgroundColor::BottomSheetBackground)
                        .border_radius(Corner::TopLeft, 24.px())
                        .border_radius(Corner::TopRight, 24.px())
                        .position_type(FlexPosition::Absolute)
                        .position(Edge::Horizontal, 16.px())
                        .position(Edge::Top, 88.px())
                        .position(Edge::Bottom, 0.px()),
                ),
            )
            .build()
    }
}
