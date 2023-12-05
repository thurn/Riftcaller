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

use core_ui::component::Component;
use core_ui::design::{BackgroundColor, FontSize};
use core_ui::prelude::*;
use core_ui::style::Corner;
use core_ui::text::Text;
use game_data::delegate_data::CardInfoElementKind;
use protos::riftcaller::{FlexAlign, FlexJustify, TextAlign, WhiteSpace};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CardInfoElement {
    pub text: String,
    pub kind: CardInfoElementKind,
}

impl CardInfoElement {
    /// Creates a new 'information' CardInfoElement
    pub fn new(s: impl Into<String>) -> Self {
        Self { text: s.into(), kind: CardInfoElementKind::Informative }
    }

    pub fn kind(mut self, kind: CardInfoElementKind) -> Self {
        self.kind = kind;
        self
    }
}

#[derive(Debug)]
pub struct SupplementalCardInfo {
    info: Vec<CardInfoElement>,
}

impl SupplementalCardInfo {
    pub fn new(info: Vec<CardInfoElement>) -> Self {
        Self { info }
    }
}

impl Component for SupplementalCardInfo {
    fn build(self) -> Option<Node> {
        let mut result = Column::new("SupplementalInfo").style(
            Style::new()
                .align_items(FlexAlign::FlexStart)
                .justify_content(FlexJustify::FlexStart)
                .margin(Edge::Horizontal, 16.px())
                .max_width(600.px())
                .max_height(600.px()),
        );

        for (i, element) in self.info.into_iter().enumerate() {
            let color = match element.kind {
                CardInfoElementKind::Informative => BackgroundColor::CardInfo,
                CardInfoElementKind::PositiveEffect => BackgroundColor::PositiveCardInfo,
                CardInfoElementKind::NegativeEffect => BackgroundColor::NegativeCardInfo,
            };
            result = result.child(InfoNode::new(element.text, color).first_node(i == 0));
        }

        result.build()
    }
}

/// A single node of supplemental card info
#[derive(Debug)]
pub struct InfoNode {
    first_node: bool,
    text: String,
    background_color: BackgroundColor,
}

impl InfoNode {
    pub fn new(text: impl Into<String>, background_color: BackgroundColor) -> Self {
        Self { first_node: false, text: text.into(), background_color }
    }

    pub fn first_node(mut self, first_node: bool) -> Self {
        self.first_node = first_node;
        self
    }
}

impl Component for InfoNode {
    fn build(self) -> Option<Node> {
        Row::new("InfoNode")
            .style(
                Style::new()
                    .margin(Edge::Bottom, 4.px())
                    .margin(Edge::Top, if self.first_node { 0.px() } else { 4.px() })
                    .background_color(self.background_color)
                    .border_radius(Corner::All, 12.px())
                    .justify_content(FlexJustify::Center)
                    .align_items(FlexAlign::Center),
            )
            .child(
                Text::new(self.text)
                    .font_size(FontSize::SupplementalInfo)
                    .white_space(WhiteSpace::Normal)
                    .text_align(TextAlign::MiddleLeft)
                    .layout(Layout::new().margin(Edge::All, 16.px())),
            )
            .build()
    }
}
