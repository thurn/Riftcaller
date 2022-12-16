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

use core_ui::design::{self, Font};
use core_ui::prelude::*;
use core_ui::text::Text;
use protos::spelldawn::{BackgroundImageAutoSize, CardIcon, FlexAlign, FlexJustify, FlexPosition};

use crate::CardHeight;

pub struct DeckCardIcon {
    icon: CardIcon,
    name: String,
    card_height: CardHeight,
    layout: Layout,
}

impl DeckCardIcon {
    pub fn new(icon: CardIcon, card_height: CardHeight) -> Self {
        Self { icon, name: "Icon".to_string(), card_height, layout: Layout::default() }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
}

impl Component for DeckCardIcon {
    fn build(self) -> Option<Node> {
        Column::new(self.name)
            .style(
                self.layout
                    .to_style()
                    .position_type(FlexPosition::Absolute)
                    .justify_content(FlexJustify::Center)
                    .align_items(FlexAlign::Center)
                    .background_image_option(self.icon.background)
                    .height(self.card_height.dim(17.0))
                    .background_image_auto_size(BackgroundImageAutoSize::FromHeight),
            )
            .child(
                Text::new(self.icon.text.unwrap_or_default())
                    .raw_font_size(self.card_height.dim(13.0))
                    .font(Font::CardIcon)
                    .outline_color(design::TEXT_OUTLINE)
                    .outline_width(2.px()),
            )
            .build()
    }
}
