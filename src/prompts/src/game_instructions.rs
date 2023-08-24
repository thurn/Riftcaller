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

use core_ui::design::{BackgroundColor, FontSize};
use core_ui::prelude::*;
use core_ui::style::Corner;
use core_ui::text::Text;
use protos::spelldawn::{FlexPosition, WhiteSpace};

pub struct GameInstructions {
    text: String,
    metatext: Option<String>,
}

impl GameInstructions {
    pub fn new(text: String) -> Self {
        Self { text, metatext: None }
    }

    pub fn metatext(mut self, metatext: impl Into<String>) -> Self {
        self.metatext = Some(metatext.into());
        self
    }
}

impl Component for GameInstructions {
    fn build(self) -> Option<Node> {
        Column::new("GameInstructions")
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::Top, 8.px())
                    .position(Edge::Left, 50.pct())
                    .translate((-50).pct(), 0.px())
                    .background_color(BackgroundColor::GameInstructionsBackground)
                    .padding(Edge::All, 16.px())
                    .max_width(50.pct())
                    .border_radius(Corner::All, 16.px()),
            )
            .child(
                Text::new(self.text)
                    .font_size(FontSize::GameInstructionsText)
                    .white_space(WhiteSpace::Normal)
                    .flex_shrink(1.0),
            )
            .child(self.metatext.map(|t| {
                Text::new(t)
                    .font_size(FontSize::GameInstructionsMetaText)
                    .white_space(WhiteSpace::Normal)
                    .flex_shrink(1.0)
            }))
            .build()
    }
}
