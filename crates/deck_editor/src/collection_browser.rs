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

use std::iter;

use core_ui::design::BLACK;
use core_ui::prelude::*;
use data::player_name::PlayerId;
use protos::spelldawn::{FlexAlign, FlexJustify};

use crate::ui_card::UICard;

#[derive(Debug)]
pub struct CollectionBrowser {
    player_id: PlayerId,
}

impl CollectionBrowser {
    pub fn new(player_id: PlayerId) -> Self {
        Self { player_id }
    }
}

fn card_row(cards: impl Iterator<Item = UICard>) -> impl Component {
    Row::new("CardRow")
        .style(
            Style::new()
                .flex_grow(1.0)
                .align_items(FlexAlign::Center)
                .justify_content(FlexJustify::Center),
        )
        .children(cards)
}

impl Component for CollectionBrowser {
    fn build(self) -> RenderResult {
        Column::new(format!("CollectionBrowser for {:?}", self.player_id))
            .style(
                Style::new()
                    .background_color(BLACK)
                    .flex_grow(1.0)
                    .margin(Edge::Horizontal, 112.px())
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center),
            )
            .child(card_row(
                iter::repeat(UICard::default().layout(Layout::new().margin(Edge::All, 16.px())))
                    .take(4),
            ))
            .child(card_row(
                iter::repeat(UICard::default().layout(Layout::new().margin(Edge::All, 16.px())))
                    .take(4),
            ))
            .build()
    }
}
