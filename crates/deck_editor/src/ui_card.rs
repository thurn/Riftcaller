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
use data::card_name::CardName;
use protos::spelldawn::StandardAction;

const CARD_ASPECT_RATIO: f32 = 0.6348214;
const CARD_HEIGHT: f32 = 36.0;

#[derive(Debug, Clone)]
pub struct UICard {
    layout: Layout,
    card_name: CardName,
    on_drop: Option<StandardAction>,
}

impl UICard {
    pub fn new(card_name: CardName) -> Self {
        Self { card_name, layout: Layout::default(), on_drop: None }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn on_drop(mut self, on_drop: StandardAction) -> Self {
        self.on_drop = Some(on_drop);
        self
    }
}

impl Component for UICard {
    fn build(self) -> RenderResult {
        Draggable::new()
            .identifiers(vec!["PlayerDecksBrowser"])
            .over_target_indicator(
                Row::new(format!("{}Title", self.card_name))
                    .style(Style::new().width(20.vw()).height(10.vh()).background_color(PINK_900)),
            )
            .on_drop(self.on_drop)
            .child(
                Column::new(self.card_name.to_string()).style(
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
