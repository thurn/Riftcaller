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

use crate::DeckCard;

#[derive(Default)]
pub struct DeckCardSlot {
    card: Option<DeckCard>,
    layout: Layout,
}

impl DeckCardSlot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn card(mut self, card: DeckCard) -> Self {
        self.card = Some(card);
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
}

impl Component for DeckCardSlot {
    fn build(self) -> Option<Node> {
        Column::new("DeckCardSlot")
            .style(
                self.layout
                    .to_style()
                    .padding(Edge::All, 4.px())
                    .justify_content(FlexJustify::Center)
                    .align_items(FlexAlign::Center)
                    .border_color(Edge::All, GRAY_500)
                    .border_width(Edge::All, 2.px())
                    .border_radius(Corner::All, 8.px()),
            )
            .child(self.card)
            .build()
    }
}
