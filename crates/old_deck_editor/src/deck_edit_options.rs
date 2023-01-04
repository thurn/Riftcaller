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

use core_ui::button::{Button, ButtonType};
use core_ui::design::RED_900;
use core_ui::panels;
use core_ui::prelude::*;
use data::deck::Deck;
use panel_address::{CollectionBrowserFilters, OldDeckEditorData, PanelAddress};
use protos::spelldawn::FlexAlign;

use crate::deck_editor_panel::EDITOR_COLUMN_WIDTH;
use crate::deck_tile::DeckTile;

pub struct DeckEditOptions<'a> {
    deck: &'a Deck,
}

impl<'a> DeckEditOptions<'a> {
    pub fn new(deck: &'a Deck) -> Self {
        Self { deck }
    }
}

impl<'a> Component for DeckEditOptions<'a> {
    fn build(self) -> Option<Node> {
        Column::new("DeckEditOptions")
            .style(
                Style::new()
                    .background_color(RED_900)
                    .width(EDITOR_COLUMN_WIDTH.vw())
                    .align_items(FlexAlign::Stretch)
                    .padding(Edge::All, 1.vw()),
            )
            .child(DeckTile::new(self.deck))
            .child(
                Button::new("Close")
                    .button_type(ButtonType::Secondary)
                    .layout(Layout::new().margin(Edge::All, 16.px()))
                    .action(panels::set(PanelAddress::OldDeckEditor(OldDeckEditorData {
                        deck: Some(self.deck.index),
                        show_edit_options: false,
                        collection_filters: CollectionBrowserFilters::default(),
                    }))),
            )
            .child(
                Button::new("Rename")
                    .button_type(ButtonType::Secondary)
                    .layout(Layout::new().margin(Edge::All, 16.px())),
            )
            .child(
                Button::new("Delete")
                    .button_type(ButtonType::Secondary)
                    .layout(Layout::new().margin(Edge::All, 16.px())),
            )
            .build()
    }
}
