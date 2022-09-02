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

use core_ui::design::GREEN_900;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use data::player_name::PlayerId;
use protos::spelldawn::{FlexAlign, FlexJustify, FlexWrap};

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

impl Component for CollectionBrowser {
    fn build(self) -> RenderResult {
        ScrollView::new(format!("CollectionBrowser for {:?}", self.player_id))
            .style(
                Style::new()
                    .background_color(GREEN_900)
                    .flex_grow(1.0)
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center),
            )
            .child(
                Row::new("CollectionContents")
                    .style(
                        Style::new()
                            .background_color(GREEN_900)
                            .flex_grow(1.0)
                            .align_items(FlexAlign::Center)
                            .justify_content(FlexJustify::Center)
                            .wrap(FlexWrap::Wrap),
                    )
                    .children(
                        iter::repeat(
                            UICard::default().layout(Layout::new().margin(Edge::All, 16.px())),
                        )
                        .take(20),
                    ),
            )
            .build()
    }
}
