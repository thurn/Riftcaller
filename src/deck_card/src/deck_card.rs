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

pub mod deck_card_slot;

pub const CARD_ASPECT_RATIO: f32 = 0.6348214;

use adapters::response_builder::{ResponseBuilder, ResponseState};
use core_ui::draggable::Draggable;
use core_ui::prelude::*;
use core_ui::style;
use display::card_sync;
use game_data::card_name::CardName;
use game_data::card_view_context::CardViewContext;
use game_data::primitives::Side;
use protos::spelldawn::studio_display::Display;
use protos::spelldawn::{
    CardIcon, CardView, Dimension, FlexAlign, FlexPosition, ImageScaleMode, StudioDisplay,
};

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
    quantity: u32,
    layout: Layout,
    draggable: Option<Draggable>,
}

impl DeckCard {
    pub fn new(name: CardName) -> Self {
        Self {
            name,
            height: CardHeight::vh(36.0),
            quantity: 1,
            layout: Layout::default(),
            draggable: None,
        }
    }

    pub fn height(mut self, height: impl Into<CardHeight>) -> Self {
        self.height = height.into();
        self
    }

    pub fn quantity(mut self, quantity: u32) -> Self {
        self.quantity = quantity;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn draggable(mut self, draggable: Option<Draggable>) -> Self {
        self.draggable = draggable;
        self
    }
}

fn add_quantity(quantity: u32, mut view: CardView) -> CardView {
    let mut icons = view.card_icons.unwrap_or_default();
    icons.top_right_icon = Some(CardIcon {
        background: Some(style::sprite("Sprites/QuantityBackground")),
        text: Some(format!("{quantity}x")),
        background_scale: None,
    });
    view.card_icons = Some(icons);
    view
}

impl Component for DeckCard {
    fn build(self) -> Option<Node> {
        let definition = rules::get(self.name);
        let response_builder = ResponseBuilder::new(
            Side::Champion,
            ResponseState { animate: false, is_final_update: true },
        );
        let context = CardViewContext::Default(definition);

        let result = Column::new(element_names::deck_card(self.name))
            .style(
                self.layout
                    .to_style()
                    .align_items(FlexAlign::Center)
                    .height(self.height.dim(100.0))
                    .width(self.height.dim(100.0 * CARD_ASPECT_RATIO)),
            )
            .child(
                Row::new("Card").style(
                    Style::new()
                        // We zoom the size of this and offset it slightly
                        // because the camera adds extra space around the
                        // captured image
                        .position_type(FlexPosition::Absolute)
                        .height(self.height.dim(110.0))
                        .width(self.height.dim(110.0 * CARD_ASPECT_RATIO))
                        .background_image_scale_mode(ImageScaleMode::ScaleAndCrop)
                        .background_display(StudioDisplay {
                            display: Some(Display::Card(Box::new(add_quantity(
                                self.quantity,
                                card_sync::card_view(&response_builder, &context)
                                    .expect("Error building CardView"),
                            )))),
                        })
                        .position(Edge::Top, self.height.dim(-6.0))
                        .position(Edge::Left, self.height.dim(-2.5)),
                ),
            );

        if let Some(draggable) = self.draggable {
            draggable.child(result).build()
        } else {
            result.build()
        }
    }
}
