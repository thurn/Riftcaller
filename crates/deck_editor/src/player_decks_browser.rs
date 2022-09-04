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
use data::player_name::PlayerId;
use protos::spelldawn::FlexDirection;

use crate::deck_editor_panel::EDITOR_COLUMN_WIDTH;

#[derive(Debug)]
pub struct PlayerDecksBrowser {
    player_id: PlayerId,
}

impl PlayerDecksBrowser {
    pub fn new(player_id: PlayerId) -> Self {
        Self { player_id }
    }
}

impl Component for PlayerDecksBrowser {
    fn build(self) -> RenderResult {
        DropTarget::new("player_decks_browser")
            .style(Style::new().flex_direction(FlexDirection::Row))
            .child(
                Column::new(format!("PlayerDecksBrowser for {:?}", self.player_id))
                    .style(Style::new().background_color(RED_900).width(EDITOR_COLUMN_WIDTH.vw())),
            )
            .build()
    }
}
