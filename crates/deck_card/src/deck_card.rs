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

pub mod card_nameplate;

pub const CARD_ASPECT_RATIO: f32 = 0.6348214;

/// Card height as a percentage of the height of the viewport. Intended to allow
/// two rows of cards to be displayed with room for additional UI elements.
pub const CARD_HEIGHT: f32 = 36.0;

use core_ui::prelude::*;
use data::card_name::CardName;
use protos::spelldawn::{BackgroundImageAutoSize, Dimension, FlexAlign};

use crate::card_nameplate::CardNameplate;

/// Abstraction representing the height of a card, allowing other measurments to
/// be scaled proportionately.
#[derive(Clone, Copy, Debug)]
pub struct CardHeight(f32);

impl CardHeight {
    pub fn vh(value: f32) -> Self {
        Self(value)
    }

    /// Returns a [Dimension] scaled as a fraction of the card height as
    /// percentage out of 100.
    pub fn dim(&self, p: f32) -> Dimension {
        (self.0 * (p / 100.0)).vh().into()
    }
}

pub struct DeckCard {
    name: CardName,
    height: CardHeight,
    layout: Layout,
}

impl DeckCard {
    pub fn new(name: CardName) -> Self {
        Self { name, height: CardHeight::vh(36.0), layout: Layout::default() }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn height(mut self, height: impl Into<CardHeight>) -> Self {
        self.height = height.into();
        self
    }
}

impl Component for DeckCard {
    fn build(self) -> Option<Node> {
        let definition = rules::get(self.name);

        Column::new(self.name.to_string())
            .style(
                self.layout
                    .to_style()
                    .align_items(FlexAlign::Center)
                    .background_image(assets::card_frame(definition.school))
                    .height(self.height.dim(100.0))
                    .background_image_auto_size(BackgroundImageAutoSize::FromHeight),
            )
            .child(CardNameplate::new(definition, self.height))
            .build()
    }
}
