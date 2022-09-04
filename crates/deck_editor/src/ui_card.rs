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

use core_ui::design::{ORANGE_900, PINK_900};
use core_ui::draggable::Draggable;
use core_ui::prelude::*;

const CARD_ASPECT_RATIO: f32 = 0.6348214;
const CARD_HEIGHT: f32 = 36.0;

#[derive(Debug, Default, Clone)]
pub struct UICard {
    layout: Layout,
}

impl UICard {
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
}

impl Component for UICard {
    fn build(self) -> RenderResult {
        Draggable::new(vec!["player_decks_browser"])
            .over_target_indicator(
                Row::new("CardTitle")
                    .style(Style::new().width(20.vw()).height(10.vh()).background_color(PINK_900)),
            )
            .child(
                Column::new("UICard").style(
                    self.layout
                        .to_style()
                        .background_color(ORANGE_900)
                        .width((CARD_HEIGHT * CARD_ASPECT_RATIO).vh())
                        .height(CARD_HEIGHT.vh()),
                ),
            )
            .build()
    }
}
