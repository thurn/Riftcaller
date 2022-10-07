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

use core_ui::actions::{InterfaceAction, NoAction};
use core_ui::design::{FontSize, PINK_900};
use core_ui::prelude::*;
use core_ui::text::Text;
use data::deck::Deck;
use protos::spelldawn::{FlexAlign, FlexDirection, FlexJustify};

use crate::deck_editor_panel::EDITOR_COLUMN_WIDTH;

pub struct DeckTile<'a> {
    deck: &'a Deck,
    layout: Layout,
    action: Box<dyn InterfaceAction>,
}

impl<'a> DeckTile<'a> {
    pub fn new(deck: &'a Deck) -> Self {
        Self { deck, layout: Layout::default(), action: Box::new(NoAction {}) }
    }

    pub fn action(mut self, action: impl InterfaceAction + 'static) -> Self {
        self.action = Box::new(action);
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
}

impl<'a> Component for DeckTile<'a> {
    fn build(self) -> Option<Node> {
        Row::new(self.deck.name.clone())
            .style(
                Style::new()
                    .height(132.px())
                    .width((EDITOR_COLUMN_WIDTH - 1).vw())
                    .flex_direction(FlexDirection::Row)
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center)
                    .background_color(PINK_900)
                    .margin(Edge::Vertical, 8.px()),
            )
            .on_click(self.action.as_client_action())
            .child(
                Column::new("DeckName")
                    .style(
                        Style::new()
                            .flex_grow(1.0)
                            .justify_content(FlexJustify::Center)
                            .padding(Edge::All, 8.px())
                            .align_items(FlexAlign::FlexStart),
                    )
                    .child(
                        Text::new(self.deck.name.clone(), FontSize::CardName)
                            .layout(Layout::new().margin(Edge::All, 0.px())),
                    ),
            )
            .build()
    }
}
