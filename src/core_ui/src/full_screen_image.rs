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

use constants::ui_constants;
use protos::riftcaller::{FlexAlign, FlexJustify, FlexPosition, ImageScaleMode, SpriteAddress};

use crate::component::EmptyComponent;
use crate::design;
use crate::design::BackgroundColor;
use crate::prelude::*;

/// Renders a full-screen image behind some content
pub struct FullScreenImage {
    image: SpriteAddress,
    content: Box<dyn ComponentObject>,
    disable_overlay: bool,
}

impl Default for FullScreenImage {
    fn default() -> Self {
        Self {
            image: SpriteAddress::default(),
            content: Box::new(EmptyComponent {}),
            disable_overlay: false,
        }
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

    /// Disabling showing a translucent black overlay on top of the image
    pub fn disable_overlay(mut self, disable_overlay: bool) -> Self {
        self.disable_overlay = disable_overlay;
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
                            .background_color(if self.disable_overlay {
                                design::TRANSPARENT
                            } else {
                                BackgroundColor::TilePanelOverlay.into()
                            })
                            .position(Edge::All, 0.px())
                            .align_items(FlexAlign::Stretch)
                            .justify_content(FlexJustify::Center)
                            .padding(Edge::Top, ui_constants::NAVBAR_HEIGHT.px())
                            .padding(Edge::Left, 1.safe_area_left())
                            .padding(Edge::Right, 1.safe_area_right())
                            .padding(Edge::Bottom, 1.safe_area_bottom()),
                    )
                    .child_boxed(self.content),
            )
            .build()
    }
}
