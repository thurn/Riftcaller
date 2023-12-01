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

use protos::riftcaller::{FlexPosition, ImageScaleMode, Node, SpriteAddress};

use crate::component::Component;
use crate::flexbox::Row;
use crate::prelude::*;
use crate::style;

pub struct FullScreenLoading {
    image: SpriteAddress,
}

impl FullScreenLoading {
    pub fn new(image: impl Into<String>) -> Self {
        Self { image: style::sprite(image.into()) }
    }
}

impl Component for FullScreenLoading {
    fn build(self) -> Option<Node> {
        Row::new("AdventureLoading")
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::All, 0.px())
                    .background_image(self.image)
                    .background_image_scale_mode(ImageScaleMode::ScaleAndCrop),
            )
            .build()
    }
}
