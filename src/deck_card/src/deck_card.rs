// Copyright © Riftcaller 2021-present

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

use adapters::response_builder::{ResponseBuilder, ResponseState};
use card_definition_data::card_view_context::CardViewContext;
use card_definition_data::cards;
use core_data::game_primitives::Milliseconds;
use core_ui::draggable::Draggable;
use core_ui::prelude::*;
use core_ui::style;
use display::card_sync;
use game_data::card_name::CardVariant;
use protos::riftcaller::game_command::Command;
use protos::riftcaller::studio_appear_effect::StudioAppear;
use protos::riftcaller::studio_display::Display;
use protos::riftcaller::{
    CardIcon, CardView, Dimension, FlexAlign, FlexPosition, ImageScaleMode, InfoZoomCommand,
    StudioAppearEffect, StudioDisplay, StudioDisplayCard,
};

pub mod deck_card_slot;

pub const CARD_ASPECT_RATIO: f32 = 0.6348214;

/// Abstraction representing the height of a card, allowing other measurements
/// to be scaled proportionately.
#[derive(Clone, Copy, Debug)]
pub struct CardHeight(f32);

/// Builds a new default [CardView] for a card based on its [CardVariant].
pub fn card_view_for_variant(variant: CardVariant) -> CardView {
    let definition = cards::get(variant);
    let response_builder = ResponseBuilder::new(
        definition.side,
        ResponseState { animate: false, is_final_update: true, display_preference: None },
    );
    let context = CardViewContext::Default(definition);
    card_sync::card_view(&response_builder, &context)
}

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
    variant: CardVariant,
    height: CardHeight,
    quantity: Option<u32>,
    layout: Layout,
    reveal_delay: Option<Milliseconds>,
    draggable: Option<Draggable>,
}

impl DeckCard {
    pub fn new(variant: CardVariant) -> Self {
        Self {
            variant,
            height: CardHeight::vh(36.0),
            quantity: None,
            layout: Layout::default(),
            reveal_delay: None,
            draggable: None,
        }
    }

    pub fn height(mut self, height: impl Into<CardHeight>) -> Self {
        self.height = height.into();
        self
    }

    pub fn quantity(mut self, quantity: Option<u32>) -> Self {
        self.quantity = quantity;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    /// Optionally, wait a fixed amount of time after display before flipping
    /// the card face up.
    pub fn reveal_delay(mut self, delay: Option<Milliseconds>) -> Self {
        self.reveal_delay = delay;
        self
    }

    pub fn draggable(mut self, draggable: Option<Draggable>) -> Self {
        self.draggable = draggable;
        self
    }
}

fn studio_display_card(
    quantity: Option<u32>,
    reveal_delay: Option<Milliseconds>,
    mut view: CardView,
) -> StudioDisplayCard {
    if let Some(quantity) = quantity {
        let mut icons = view.card_icons.unwrap_or_default();
        icons.top_right_icon = Some(CardIcon {
            background: Some(style::sprite("Sprites/QuantityBackground")),
            text: Some(format!("{quantity}x")),
            background_scale: None,
        });
        view.card_icons = Some(icons);
    }

    let appear_effects = if let Some(delay) = reveal_delay {
        view.revealed_to_viewer = false;
        vec![StudioAppearEffect {
            delay: Some(adapters::time_value(delay)),
            studio_appear: Some(StudioAppear::SetRevealed(true)),
        }]
    } else {
        vec![]
    };

    StudioDisplayCard { card: Some(Box::new(view)), appear_effects }
}

impl Component for DeckCard {
    fn build(self) -> Option<Node> {
        let card_view = card_view_for_variant(self.variant);
        Column::new(element_names::deck_card(self.variant))
            .style(
                self.layout
                    .to_style()
                    .align_items(FlexAlign::Center)
                    .height(self.height.dim(100.0))
                    .width(self.height.dim(100.0 * CARD_ASPECT_RATIO)),
            )
            .on_mouse_enter(Command::InfoZoom(InfoZoomCommand {
                show: true,
                card: Some(card_view.clone()),
            }))
            .on_mouse_leave(Command::InfoZoom(InfoZoomCommand { show: false, card: None }))
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
                            display: Some(Display::Card(Box::new(studio_display_card(
                                self.quantity,
                                self.reveal_delay,
                                card_view,
                            )))),
                        })
                        .position(Edge::Top, self.height.dim(-6.0))
                        .position(Edge::Left, self.height.dim(-2.5)),
                ),
            )
            .build()
    }
}
