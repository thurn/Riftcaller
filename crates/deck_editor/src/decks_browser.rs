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
use core_ui::prelude::*;
use data::player_name::PlayerId;

#[derive(Debug)]
pub struct DecksBrowser {
    player_id: PlayerId,
}

impl DecksBrowser {
    pub fn new(player_id: PlayerId) -> Self {
        Self { player_id }
    }
}

impl Component for DecksBrowser {
    fn build(self) -> RenderResult {
        Column::new(format!("DecksBrowser for {:?}", self.player_id))
            .style(Style::new().background_color(RED_900).width(350.px()))
            .build()
    }
}
