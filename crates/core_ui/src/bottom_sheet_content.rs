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

use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition, InterfacePanelAddress};

use crate::button::{IconButton, IconButtonType};
use crate::component::EmptyComponent;
use crate::design::FontSize;
use crate::prelude::*;
use crate::text::Text;
use crate::{icons, panel};

pub enum BottomSheetButtonType {
    /// Close the bottom sheet
    Close,

    /// Navigate back to the specified panel
    Back(InterfacePanelAddress),
}

pub struct BottomSheetContent {
    content: Box<dyn ComponentObject>,
    title: Option<String>,
    button_type: BottomSheetButtonType,
}

impl Default for BottomSheetContent {
    fn default() -> Self {
        Self {
            content: Box::new(EmptyComponent),
            title: None,
            button_type: BottomSheetButtonType::Close,
        }
    }
}

impl BottomSheetContent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(mut self, content: impl Component + 'static) -> Self {
        self.content = Box::new(content);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn button_type(mut self, button_type: BottomSheetButtonType) -> Self {
        self.button_type = button_type;
        self
    }
}

impl Component for BottomSheetContent {
    fn build(self) -> Option<Node> {
        Column::new(format!("{}Sheet", self.title.clone().unwrap_or_default()))
            .style(
                Style::new()
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center)
                    .flex_grow(1.0)
                    .width(100.pct()),
            )
            .child(
                Row::new("TitleRow")
                    .style(Style::new().width(100.pct()).justify_content(FlexJustify::Center))
                    .child(
                        IconButton::new(match self.button_type {
                            BottomSheetButtonType::Close => icons::CLOSE,
                            BottomSheetButtonType::Back(_) => icons::BACK,
                        })
                        .action(match self.button_type {
                            BottomSheetButtonType::Close => panel::close_bottom_sheet(),
                            BottomSheetButtonType::Back(address) => {
                                panel::pop_to_bottom_sheet(address)
                            }
                        })
                        .button_type(IconButtonType::SecondaryLarge)
                        .layout(
                            Layout::new()
                                .position_type(FlexPosition::Absolute)
                                .position(Edge::Left, 20.px())
                                .position(Edge::Top, 20.px()),
                        ),
                    )
                    .child(
                        Text::new(self.title.unwrap_or_default(), FontSize::PanelTitle)
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    ),
            )
            .child(
                Column::new("SheetContent")
                    .style(Style::new().flex_grow(1.0).justify_content(FlexJustify::Center))
                    .child_boxed(self.content),
            )
            .build()
    }
}
