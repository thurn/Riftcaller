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

use core_ui::design::RED_900;
use core_ui::drop_target::DropTarget;
use core_ui::prelude::*;
use data::player_data::PlayerData;
use data::primitives::DeckId;
use protos::spelldawn::{FlexAlign, FlexDirection};

use crate::deck_editor_panel::EDITOR_COLUMN_WIDTH;

/// Displays the cards contained within a single deck
#[allow(dead_code)]
#[derive(Debug)]
pub struct DeckCardList<'a> {
    player: &'a PlayerData,
    deck_id: DeckId,
}

impl<'a> DeckCardList<'a> {
    pub fn new(player: &'a PlayerData, deck_id: DeckId) -> Self {
        DeckCardList { player, deck_id }
    }
}

impl<'a> Component for DeckCardList<'a> {
    fn build(self) -> Option<Node> {
        DropTarget::new("DeckCardList")
            .style(
                Style::new()
                    .flex_direction(FlexDirection::Column)
                    .background_color(RED_900)
                    .width(EDITOR_COLUMN_WIDTH.vw())
                    .align_items(FlexAlign::Center)
                    .padding(Edge::All, 1.vw()),
            )
            .build()
    }
}
