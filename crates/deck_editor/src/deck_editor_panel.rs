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

use core_ui::design::BackgroundColor;
use core_ui::prelude::*;
use data::player_name::PlayerId;
use panel_address::DeckEditorData;

use crate::collection_browser::CollectionBrowser;
use crate::collection_controls::CollectionControls;
use crate::decks_browser::DecksBrowser;

#[derive(Debug)]
pub struct DeckEditorPanel {
    player_id: PlayerId,
    data: DeckEditorData,
}

impl DeckEditorPanel {
    pub fn new(player_id: PlayerId, data: DeckEditorData) -> Self {
        Self { player_id, data }
    }
}

impl Component for DeckEditorPanel {
    fn build(self) -> RenderResult {
        Row::new(format!("DeckEditorPanel {:?}", self.data.deck))
            .style(
                Style::new()
                    .background_color(BackgroundColor::DeckEditorPanel)
                    .width(100.pct())
                    .height(100.pct()),
            )
            .child(
                Column::new("Collection")
                    .style(Style::new().flex_grow(1.0))
                    .child(CollectionControls::new(self.player_id))
                    .child(CollectionBrowser::new(self.player_id)),
            )
            .child(DecksBrowser::new(self.player_id))
            .build()
    }
}
