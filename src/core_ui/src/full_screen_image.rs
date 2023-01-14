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

use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition, ImageScaleMode, SpriteAddress};

use crate::component::EmptyComponent;
use crate::design::BackgroundColor;
use crate::prelude::*;

/// Renders a full-screen image containing a text prompt and some arbitrary
/// content.
pub struct FullScreenImage {
    image: SpriteAddress,
    content: Box<dyn ComponentObject>,
}

impl Default for FullScreenImage {
    fn default() -> Self {
        Self { image: SpriteAddress::default(), content: Box::new(EmptyComponent {}) }
    }
}

impl FullScreenImage {
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

impl Component for FullScreenImage {
    fn build(self) -> Option<Node> {
        Row::new("ImagePanel")
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::All, 0.px())
                    .background_image_scale_mode(ImageScaleMode::ScaleAndCrop)
                    .background_image(self.image),
            )
            .child(
                Column::new("FullImageContent")
                    .style(
                        Style::new()
                            .position_type(FlexPosition::Absolute)
                            .background_color(BackgroundColor::TilePanelOverlay)
                            .position(Edge::All, 0.px())
                            .align_items(FlexAlign::Stretch)
                            .justify_content(FlexJustify::Center)
                            .padding(Edge::Top, constants::NAVBAR_HEIGHT.px())
                            .padding(Edge::Left, 1.safe_area_left())
                            .padding(Edge::Right, 1.safe_area_right())
                            .padding(Edge::Bottom, 1.safe_area_bottom()),
                    )
                    .child_boxed(self.content),
            )
            .build()
    }
}
