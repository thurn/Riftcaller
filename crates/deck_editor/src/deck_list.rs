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
use core_ui::design::{FontSize, RED_900};
use core_ui::panel;
use core_ui::prelude::*;
use core_ui::text::Text;
use data::player_data::PlayerData;
use panel_address::PanelAddress;
use protos::spelldawn::{FlexAlign, FlexDirection};

use crate::deck_name::DeckName;
use crate::editor_column_scroll::EditorColumnScroll;

/// Displays the decks owned by a player
#[derive(Debug)]
pub struct DeckList<'a> {
    player: &'a PlayerData,
}

impl<'a> DeckList<'a> {
    pub fn new(player: &'a PlayerData) -> Self {
        DeckList { player }
    }
}

impl<'a> Component for DeckList<'a> {
    fn build(self) -> Option<Node> {
        let mut decks = self.player.decks.iter().collect::<Vec<_>>();
        decks.sort_by_key(|d| (d.side, rules::get(d.identity).school, d.identity.displayed_name()));
        Column::new("DeckList")
            .style(Style::new().background_color(RED_900))
            .child(Text::new("Decks", FontSize::PanelTitle))
            .child(
                EditorColumnScroll::new()
                    .child(
                        Button::new("Create Deck")
                            .button_type(ButtonType::Primary)
                            .layout(Layout::new().margin(Edge::All, 16.px()))
                            .action(panel::open_bottom_sheet(PanelAddress::CreateDeck)),
                    )
                    .child(
                        Column::new("Decks")
                            .style(
                                Style::new()
                                    .flex_direction(FlexDirection::Column)
                                    .align_items(FlexAlign::Center)
                                    .padding(Edge::All, 1.vw()),
                            )
                            .children(decks.into_iter().map(DeckName::new)),
                    ),
            )
            .build()
    }
}
