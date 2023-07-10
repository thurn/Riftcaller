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

use std::collections::HashMap;

use core_ui::button::{IconButton, IconButtonType};
use core_ui::full_screen_image::FullScreenImage;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::{icons, style};
use game_data::card_name::CardName;
use game_data::deck::Deck;
use game_data::primitives::DeckId;
use panel_address::{CollectionBrowserFilters, DeckEditorData, Panel, PanelAddress, PlayerPanel};
use player_data::PlayerState;
use protos::spelldawn::FlexJustify;
use screen_overlay::ScreenOverlay;

use crate::card_list::CardList;
use crate::collection_browser::{self, CollectionBrowser};

pub const EDITOR_COLUMN_WIDTH: i32 = 25;

pub struct DeckEditorPanel<'a> {
    pub player: &'a PlayerState,
    pub data: DeckEditorData,
    pub deck: &'a Deck,
    pub collection: &'a HashMap<CardName, u32>,
}

impl<'a> DeckEditorPanel<'a> {
    fn page_control(&self, show: bool, icon: impl Into<String>, subtract: bool) -> impl Component {
        Column::new("PageControls")
            .style(
                Style::new()
                    .min_width(96.px())
                    .height(100.pct())
                    .flex_shrink(0.0)
                    .flex_grow(1.0)
                    .justify_content(FlexJustify::Center),
            )
            .child(show.then(|| {
                IconButton::new(icon).button_type(IconButtonType::NavBrown).action(
                    Panels::open(PlayerPanel::DeckEditor(DeckEditorData {
                        deck_id: DeckId::Adventure,
                        collection_filters: CollectionBrowserFilters {
                            offset: if subtract {
                                self.data.collection_filters.offset - 8
                            } else {
                                self.data.collection_filters.offset + 8
                            },
                        },
                    }))
                    .and_close(self.address())
                    .wait_to_load(true),
                )
            }))
    }
}

impl<'a> Panel for DeckEditorPanel<'a> {
    fn address(&self) -> PanelAddress {
        PlayerPanel::DeckEditor(self.data).into()
    }

    fn screen_overlay(&self) -> Option<Node> {
        ScreenOverlay::new(self.player)
            .show_deck_button(false)
            .show_close_button(Panels::close(self.address()))
            .build()
    }
}

impl<'a> Component for DeckEditorPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImage::new()
            .image(style::sprite(
                "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Library/SceneryLibrary_inside_1",
            ))
            .content(
                Row::new("DeckEditorPanel")
                    .child(self.page_control(
                        self.data.collection_filters.offset >= 8,
                        icons::PREVIOUS_PAGE,
                        true,
                    ))
                    .child(Column::new("Collection").child(CollectionBrowser {
                        player: self.player,
                        deck: self.deck,
                        collection: self.collection,
                        filters: self.data.collection_filters,
                    }))
                    .child(
                        self.page_control(
                            self.data.collection_filters.offset + 8
                                < collection_browser::get_matching_cards(
                                    self.collection,
                                    self.data.collection_filters,
                                )
                                .count(),
                            icons::NEXT_PAGE,
                            false,
                        ),
                    )
                    .child(CardList { deck: self.deck }),
            )
            .build()
    }
}
