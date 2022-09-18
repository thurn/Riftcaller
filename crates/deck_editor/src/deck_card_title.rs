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

use core_ui::design::{FontSize, PINK_900};
use core_ui::draggable::Draggable;
use core_ui::prelude::*;
use core_ui::text::Text;
use data::card_name::CardName;
use protos::spelldawn::{FlexAlign, FlexJustify, StandardAction, TextAlign};

use crate::deck_card::DeckCard;

#[derive(Debug)]
pub struct DeckCardTitle {
    layout: Layout,
    card_name: CardName,
    on_drop: Option<StandardAction>,
    count: Option<u32>,
}

impl DeckCardTitle {
    pub fn new(card_name: CardName) -> Self {
        Self { card_name, layout: Layout::default(), on_drop: None, count: None }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn on_drop(mut self, on_drop: Option<StandardAction>) -> Self {
        self.on_drop = on_drop;
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }
}

impl Component for DeckCardTitle {
    fn build(self) -> Option<Node> {
        Draggable::new(format!("{}Title", self.card_name))
            .drop_targets(vec!["CollectionBrowser"])
            .over_target_indicator(move || DeckCard::new(self.card_name).build())
            .on_drop(self.on_drop)
            .horizontal_drag_start_distance(100)
            .remove_original(if let Some(v) = self.count { v < 2 } else { false })
            .style(
                Style::new()
                    .width(20.vw())
                    .height(10.vh())
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center)
                    .background_color(PINK_900)
                    .margin(Edge::All, 1.vh()),
            )
            .child(
                Text::new(self.card_name.displayed_name(), FontSize::CardName)
                    .text_align(TextAlign::MiddleCenter),
            )
            .build()
    }
}
