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

use core_ui::design::{FontSize, ORANGE_900};
use core_ui::draggable::Draggable;
use core_ui::prelude::*;
use core_ui::text::Text;
use data::card_name::CardName;
use protos::spelldawn::{StandardAction, TextAlign};

use crate::deck_editor_card_title::DeckEditorCardTitle;

pub const CARD_ASPECT_RATIO: f32 = 0.6348214;

/// Card height as a percentage of the height of the viewport. Intended to allow
/// two rows of cards to be displayed with room for additional UI elements.
pub const CARD_HEIGHT: f32 = 36.0;

/// Displays a single named card as it appears in the UI
#[derive(Debug)]
pub struct DeckEditorCard {
    layout: Layout,
    card_name: CardName,
    on_drop: Option<StandardAction>,
}

impl DeckEditorCard {
    pub fn new(card_name: CardName) -> Self {
        Self { card_name, layout: Layout::default(), on_drop: None }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn on_drop(mut self, on_drop: Option<StandardAction>) -> Self {
        self.on_drop = on_drop;
        self
    }
}

impl Component for DeckEditorCard {
    fn build(self) -> Option<Node> {
        Draggable::new(self.card_name.to_string())
            .drop_targets(vec!["CardList"])
            .over_target_indicator(move || DeckEditorCardTitle::new(self.card_name).build())
            .on_drop(self.on_drop)
            .style(
                self.layout
                    .to_style()
                    .background_color(ORANGE_900)
                    .width((CARD_HEIGHT * CARD_ASPECT_RATIO).vh())
                    .height(CARD_HEIGHT.vh()),
            )
            .child(
                Text::new(self.card_name.displayed_name(), FontSize::CardName)
                    .text_align(TextAlign::MiddleCenter),
            )
            .build()
    }
}
