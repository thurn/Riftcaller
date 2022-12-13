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

use core_ui::component::EmptyComponent;
use core_ui::design::BackgroundColor;
use core_ui::prelude::*;
use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition, SpriteAddress};

/// Renders a full-screen image containing a text prompt and some arbitrary
/// content.
pub struct FullScreenImagePanel {
    image: SpriteAddress,
    content: Box<dyn Component>,
}

impl Default for FullScreenImagePanel {
    fn default() -> Self {
        Self { image: SpriteAddress::default(), content: Box::new(EmptyComponent {}) }
    }
}

impl FullScreenImagePanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn image(mut self, image: SpriteAddress) -> Self {
        self.image = image;
        self
    }

    pub fn content(mut self, content: impl Component + 'static) -> Self {
        self.content = Box::new(content);
        self
    }
}

impl Component for FullScreenImagePanel {
    fn build(self) -> Option<Node> {
        Row::new("ImagePanel")
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::All, 0.px())
                    .background_image(self.image),
            )
            .child(
                Column::new("FullImageContent")
                    .style(
                        Style::new()
                            .position_type(FlexPosition::Absolute)
                            .background_color(BackgroundColor::TilePanelOverlay)
                            .position(Edge::All, 0.px())
                            .justify_content(FlexJustify::Center)
                            .align_items(FlexAlign::Center)
                            .padding(Edge::All, 8.px()),
                    )
                    .child_boxed(self.content),
            )
            .build()
    }
}
