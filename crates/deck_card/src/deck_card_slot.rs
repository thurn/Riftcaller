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

use core_ui::design::GRAY_500;
use core_ui::prelude::*;
use core_ui::style::Corner;
use protos::spelldawn::{FlexAlign, FlexJustify};

use crate::{CardHeight, DeckCard};

pub struct DeckCardSlot {
    height: CardHeight,
    card: Option<DeckCard>,
    layout: Layout,
}

impl DeckCardSlot {
    pub fn new(height: CardHeight) -> Self {
        Self { height, card: None, layout: Layout::default() }
    }

    pub fn card(mut self, card: Option<DeckCard>) -> Self {
        self.card = card;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
}

impl Component for DeckCardSlot {
    fn build(self) -> Option<Node> {
        Column::new(
            self.card
                .as_ref()
                .map(|c| element_names::deck_card_slot(c.name).into())
                .unwrap_or_else(|| "DeckCardSlot".to_string()),
        )
        .style(
            self.layout
                .to_style()
                .padding(Edge::All, 4.px())
                .justify_content(FlexJustify::Center)
                .align_items(FlexAlign::Center)
                .border_color(Edge::All, GRAY_500)
                .border_width(Edge::All, 2.px())
                .height(self.height.dim(100.0))
                .width(self.height.dim(100.0 * crate::CARD_ASPECT_RATIO))
                .border_radius(Corner::All, 8.px()),
        )
        .child(self.card.map(|c| c.height(self.height)))
        .build()
    }
}
