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

use core_ui::prelude::*;
use core_ui::text::Text;
use game_data::card_definition::CardDefinition;
use game_data::card_view_context::CardViewContext;
use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition, WhiteSpace};

use crate::CardHeight;

pub struct DeckCardText<'a> {
    definition: &'a CardDefinition,
    card_height: CardHeight,
}

impl<'a> DeckCardText<'a> {
    pub fn new(definition: &'a CardDefinition, card_height: CardHeight) -> Self {
        Self { definition, card_height }
    }
}

impl<'a> Component for DeckCardText<'a> {
    fn build(self) -> Option<Node> {
        let text = rules_text::build(&CardViewContext::Default(self.definition));
        Column::new("RulesText")
            .style(
                Style::new()
                    .justify_content(FlexJustify::Center)
                    .align_items(FlexAlign::Center)
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::Top, self.card_height.dim(72.0))
                    .position(Edge::Horizontal, self.card_height.dim(10.0))
                    .position(Edge::Bottom, self.card_height.dim(4.0)),
            )
            .child(
                Text::new(text.text)
                    .raw_font_size(self.card_height.dim(3.6))
                    .white_space(WhiteSpace::Normal),
            )
            .build()
    }
}
