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
use core_ui::design::BackgroundColor;
use core_ui::prelude::*;
use core_ui::{icons, panels};
use data::deck::Deck;
use data::player_data::PlayerData;
use panel_address::{CollectionBrowserFilters, DeckEditorData, Panel, PanelAddress};
use protos::spelldawn::FlexPosition;

use crate::card_list::CardList;
use crate::collection_browser;
use crate::collection_browser::CollectionBrowser;
use crate::collection_controls::CollectionControls;
use crate::deck_edit_options::DeckEditOptions;
use crate::deck_list::DeckList;

pub const EDITOR_COLUMN_WIDTH: i32 = 25;

pub struct DeckEditorPanel<'a> {
    pub player: &'a PlayerData,
    pub open_deck: Option<&'a Deck>,
    pub data: DeckEditorData,
}

impl<'a> Panel for DeckEditorPanel<'a> {
    fn address(&self) -> PanelAddress {
        PanelAddress::DeckEditor(self.data)
    }
}

impl<'a> Component for DeckEditorPanel<'a> {
    fn build(self) -> Option<Node> {
        Row::new("OverlayBackground")
            .style(
                Style::new()
                    .background_color(BackgroundColor::SafeAreaOverlay)
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::All, 0.px()),
            )
            .child(
                Row::new("DeckEditorPanel")
                    .style(
                        Style::new()
                            .background_color(BackgroundColor::DeckEditorPanel)
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Top, 1.safe_area_top())
                            .position(Edge::Right, 1.safe_area_right())
                            .position(Edge::Bottom, 1.safe_area_bottom())
                            .position(Edge::Left, 1.safe_area_left()),
                    )
                    .child(
                        Column::new("Collection")
                            .style(Style::new().width((100 - EDITOR_COLUMN_WIDTH).vw()))
                            .child(CollectionControls::new(self.player.id, self.open_deck))
                            .child(CollectionBrowser {
                                player: self.player,
                                open_deck: self.open_deck,
                                filters: self.data.collection_filters,
                            }),
                    )
                    .child_node(match self.open_deck {
                        Some(deck) if self.data.show_edit_options => {
                            DeckEditOptions::new(deck).build()
                        }
                        Some(deck) => CardList::new(deck).build(),
                        _ => DeckList::new(self.player, self.data.collection_filters).build(),
                    })
                    .child(if self.data.collection_filters.offset < 8 {
                        None
                    } else {
                        Some(
                            IconButton::new(icons::PREVIOUS_PAGE)
                                .button_type(IconButtonType::SecondaryLarge)
                                .action(panels::set(PanelAddress::DeckEditor(DeckEditorData {
                                    deck: self.open_deck.map(|d| d.index),
                                    collection_filters: CollectionBrowserFilters {
                                        offset: self.data.collection_filters.offset - 8,
                                    },
                                    show_edit_options: false,
                                })))
                                .layout(
                                    Layout::new()
                                        .position_type(FlexPosition::Absolute)
                                        .position(Edge::Left, 1.vw())
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
                                    .action(panels::set(PanelAddress::DeckEditor(DeckEditorData {
                                        deck: self.open_deck.map(|d| d.index),
                                        collection_filters: CollectionBrowserFilters {
                                            offset: self.data.collection_filters.offset + 8,
                                        },
                                        show_edit_options: false,
                                    })))
                                    .layout(
                                        Layout::new()
                                            .position_type(FlexPosition::Absolute)
                                            .position(Edge::Right, (EDITOR_COLUMN_WIDTH + 1).vw())
                                            .position(Edge::Top, 50.pct()),
                                    ),
                            )
                        },
                    ),
            )
            .build()
    }
}
