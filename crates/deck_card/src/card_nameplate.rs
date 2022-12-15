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

use core_ui::design::FontSize;
use core_ui::prelude::*;
use core_ui::text::Text;
use data::card_definition::CardDefinition;
use protos::spelldawn::{BackgroundImageAutoSize, FlexAlign, FlexJustify};

use crate::CardHeight;

pub struct CardNameplate<'a> {
    definition: &'a CardDefinition,
    card_height: CardHeight,
}

impl<'a> CardNameplate<'a> {
    pub fn new(definition: &'a CardDefinition, card_height: CardHeight) -> Self {
        Self { definition, card_height }
    }
}

impl<'a> Component for CardNameplate<'a> {
    fn build(self) -> Option<Node> {
        Row::new("CardNameplate")
            .style(
                Style::new()
                    .background_image(assets::title_background(self.definition.config.lineage))
                    .background_image_auto_size(BackgroundImageAutoSize::FromHeight)
                    .position(Edge::Top, self.card_height.dim(-5.0))
                    .height(self.card_height.dim(22.0))
                    .justify_content(FlexJustify::Center)
                    .align_items(FlexAlign::Center),
            )
            .child(Text::new(self.definition.name.displayed_name()).font_size(FontSize::CardName))
            .build()
    }
}
