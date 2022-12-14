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

//! Renders cards as they're seen in the deck editor and adventure UI

pub const CARD_ASPECT_RATIO: f32 = 0.6348214;

/// Card height as a percentage of the height of the viewport. Intended to allow
/// two rows of cards to be displayed with room for additional UI elements.
pub const CARD_HEIGHT: f32 = 36.0;

use core_ui::prelude::*;
use data::card_name::CardName;
use protos::spelldawn::Dimension;

pub struct DeckCard {
    name: CardName,
    height: Dimension,
    layout: Layout,
}

impl DeckCard {
    pub fn new(name: CardName) -> Self {
        Self { name, height: 36.vh().into(), layout: Layout::default() }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn height(mut self, height: impl Into<Dimension>) -> Self {
        self.height = height.into();
        self
    }
}

impl Component for DeckCard {
    fn build(self) -> Option<Node> {
        let mut width = self.height.clone();
        width.value *= CARD_ASPECT_RATIO;
        let definition = rules::get(self.name);

        Column::new(self.name.to_string())
            .style(
                self.layout
                    .to_style()
                    .background_image(assets::card_frame(definition.school))
                    .width(width)
                    .height(self.height),
            )
            .build()
    }
}
