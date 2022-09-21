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
use core_ui::icons;
use core_ui::prelude::*;
use data::deck::Deck;
use data::player_data::PlayerData;
use protos::spelldawn::FlexPosition;

use crate::card_list::CardList;
use crate::collection_browser::CollectionBrowser;
use crate::collection_controls::CollectionControls;
use crate::deck_list::DeckList;

pub const EDITOR_COLUMN_WIDTH: i32 = 25;

#[derive(Debug)]
pub struct DeckEditorPanel<'a> {
    player: &'a PlayerData,
    open_deck: Option<&'a Deck>,
}

impl<'a> DeckEditorPanel<'a> {
    pub fn new(player: &'a PlayerData, open_deck: Option<&'a Deck>) -> Self {
        Self { player, open_deck }
    }
}

impl<'a> Component for DeckEditorPanel<'a> {
    fn build(self) -> Option<Node> {
        Row::new("DeckEditorPanel")
            .style(
                Style::new()
                    .background_color(BackgroundColor::DeckEditorPanel)
                    .width(100.pct())
                    .height(100.pct()),
            )
            .child(
                Column::new("Collection")
                    .style(Style::new().width((100 - EDITOR_COLUMN_WIDTH).vw()))
                    .child(CollectionControls::new(self.player.id))
                    .child(CollectionBrowser::new(self.player, self.open_deck)),
            )
            .child(self.open_deck.map(|d| CardList::new(self.player, d)))
            .child(if self.open_deck.is_none() { Some(DeckList::new(self.player)) } else { None })
            .child(
                IconButton::new(icons::PREVIOUS_PAGE)
                    .button_type(IconButtonType::SecondaryLarge)
                    .layout(
                        Layout::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Left, 1.vw())
                            .position(Edge::Top, 50.pct()),
                    ),
            )
            .child(
                IconButton::new(icons::NEXT_PAGE)
                    .button_type(IconButtonType::SecondaryLarge)
                    .layout(
                        Layout::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Right, (EDITOR_COLUMN_WIDTH + 1).vw())
                            .position(Edge::Top, 50.pct()),
                    ),
            )
            .build()
    }
}
