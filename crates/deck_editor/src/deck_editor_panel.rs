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

use core_ui::button::{IconButton, IconButtonType};
use core_ui::full_screen_image_panel::FullScreenImagePanel;
use core_ui::prelude::*;
use core_ui::{icons, style};
use data::deck::Deck;
use data::player_data::PlayerData;
use panel_address::{DeckEditorData, Panel, PanelAddress};
use protos::spelldawn::FlexPosition;

use crate::card_list::CardList;
use crate::collection_browser::{self, CollectionBrowser};

pub const EDITOR_COLUMN_WIDTH: i32 = 25;

pub struct DeckEditorPanel<'a> {
    pub player: &'a PlayerData,
    pub data: DeckEditorData,
    pub deck: &'a Deck,
}

impl<'a> Panel for DeckEditorPanel<'a> {
    fn address(&self) -> PanelAddress {
        PanelAddress::DeckEditor(self.data)
    }
}

impl<'a> Component for DeckEditorPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImagePanel::new()
            .image(style::sprite(
                "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Library/SceneryLibrary_inside_1",
            ))
            .content(
                Row::new("DeckEditorPanel")
                    .style(Style::new().width(100.pct()).height(100.pct()))
                    .child(
                        Column::new("Collection")
                            .style(Style::new().width((100 - EDITOR_COLUMN_WIDTH).vw()))
                            // .child(CollectionControls::new(self.player.id, self.open_deck))
                            .child(CollectionBrowser {
                                player: self.player,
                                deck: self.deck,
                                filters: self.data.collection_filters,
                            }),
                    )
                    .child(CardList::new(self.deck))
                    .child(if self.data.collection_filters.offset < 8 {
                        None
                    } else {
                        Some(
                            IconButton::new(icons::PREVIOUS_PAGE)
                                .button_type(IconButtonType::SecondaryLarge)
                                // .action(panels::set(PanelAddress::DeckEditor(
                                //     OldDeckEditorData {
                                //         deck: self.open_deck.map(|d| d.index),
                                //         collection_filters: CollectionBrowserFilters {
                                //             offset: self.data.collection_filters.offset - 8,
                                //         },
                                //         show_edit_options: false,
                                //     },
                                // )))
                                .layout(
                                    Layout::new()
                                        .position_type(FlexPosition::Absolute)
                                        .position(Edge::Left, 1.vw())
                                        .translate(0.pct(), (-50).pct())
                                        .position(Edge::Top, 50.pct()),
                                ),
                        )
                    })
                    .child(
                        if self.data.collection_filters.offset + 8
                            >= collection_browser::get_matching_cards(
                                self.player,
                                self.data.collection_filters,
                            )
                            .count()
                        {
                            None
                        } else {
                            Some(
                                IconButton::new(icons::NEXT_PAGE)
                                    .button_type(IconButtonType::SecondaryLarge)
                                    // .action(panels::set(PanelAddress::OldDeckEditor(
                                    //     OldDeckEditorData {
                                    //         deck: self.open_deck.map(|d| d.index),
                                    //         collection_filters: CollectionBrowserFilters {
                                    //             offset: self.data.collection_filters.offset + 8,
                                    //         },
                                    //         show_edit_options: false,
                                    //     },
                                    // )))
                                    .layout(
                                        Layout::new()
                                            .position_type(FlexPosition::Absolute)
                                            .position(Edge::Right, (EDITOR_COLUMN_WIDTH + 1).vw())
                                            .translate(0.pct(), (-50).pct())
                                            .position(Edge::Top, 50.pct()),
                                    ),
                            )
                        },
                    ),
            )
            .build()
    }
}
