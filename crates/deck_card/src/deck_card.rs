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

pub mod deck_card_icon;
pub mod deck_card_name;
pub mod deck_card_rarity;
pub mod deck_card_slot;
pub mod deck_card_text;

pub const CARD_ASPECT_RATIO: f32 = 0.6348214;

/// Card height as a percentage of the height of the viewport. Intended to allow
/// two rows of cards to be displayed with room for additional UI elements.
pub const CARD_HEIGHT: f32 = 36.0;

use core_ui::prelude::*;
use core_ui::style;
use core_ui::update_element::ElementName;
use data::card_name::CardName;
use data::text::RulesTextContext;
use protos::spelldawn::{BackgroundImageAutoSize, CardIcon, Dimension, FlexAlign, FlexPosition};
use rules_text::card_icons;

use crate::deck_card_icon::DeckCardIcon;
use crate::deck_card_name::DeckCardName;
use crate::deck_card_rarity::DeckCardRarity;
use crate::deck_card_text::DeckCardText;

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
    element_name: ElementName,
    height: CardHeight,
    quantity: u32,
    layout: Layout,
}

impl DeckCard {
    pub fn new(name: CardName) -> Self {
        Self {
            name,
            element_name: ElementName::constant(name.to_string()),
            height: CardHeight::vh(36.0),
            quantity: 1,
            layout: Layout::default(),
        }
    }

    pub fn height(mut self, height: impl Into<CardHeight>) -> Self {
        self.height = height.into();
        self
    }

    pub fn element_name(mut self, name: &ElementName) -> Self {
        self.element_name = name.clone();
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
}

impl Component for DeckCard {
    fn build(self) -> Option<Node> {
        let definition = rules::get(self.name);
        let icons = card_icons::build(&RulesTextContext::Default(definition), definition, true);

        Column::new(self.element_name)
            .style(self.layout.to_style().align_items(FlexAlign::Center))
            .child(
                Row::new("CardImage").style(
                    Style::new()
                        .position_type(FlexPosition::Absolute)
                        .background_image(adapters::sprite(&definition.image))
                        .position(Edge::Top, self.height.dim(7.0))
                        .height(self.height.dim(56.0))
                        .width(self.height.dim(56.0)),
                ),
            )
            .child(
                Row::new("CardFrame").style(
                    Style::new()
                        .height(self.height.dim(100.0))
                        .background_image_auto_size(BackgroundImageAutoSize::FromHeight)
                        .background_image(assets::card_frame(definition.school)),
                ),
            )
            .child(DeckCardName::new(definition, self.height))
            .child(DeckCardText::new(definition, self.height))
            .child(icons.top_left_icon.map(|icon| {
                DeckCardIcon::new(icon, self.height).name("TopLeftIcon").layout(
                    Layout::new()
                        .position(Edge::Left, self.height.dim(-2.0))
                        .position(Edge::Top, self.height.dim(6.0)),
                )
            }))
            .child(
                DeckCardIcon::new(
                    CardIcon {
                        background: Some(style::sprite("Sprites/QuantityBackground")),
                        text: Some(format!("{}x", self.quantity)),
                        background_scale: None,
                    },
                    self.height,
                )
                .name("TopRightIcon")
                .layout(
                    Layout::new()
                        .position(Edge::Right, self.height.dim(-2.0))
                        .position(Edge::Top, self.height.dim(6.0)),
                ),
            )
            .child(icons.bottom_left_icon.map(|icon| {
                DeckCardIcon::new(icon, self.height).name("BottomLeftIcon").layout(
                    Layout::new()
                        .position(Edge::Left, self.height.dim(-4.0))
                        .position(Edge::Bottom, self.height.dim(-6.0)),
                )
            }))
            .child(icons.bottom_right_icon.map(|icon| {
                DeckCardIcon::new(icon, self.height).name("BottomRightIcon").layout(
                    Layout::new()
                        .position(Edge::Right, self.height.dim(-4.0))
                        .position(Edge::Bottom, self.height.dim(-6.0)),
                )
            }))
            .child(DeckCardRarity::new(definition, self.height))
            .build()
    }
}
