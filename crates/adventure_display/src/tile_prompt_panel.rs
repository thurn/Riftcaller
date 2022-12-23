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

use core_ui::button::Button;
use core_ui::design::{BackgroundColor, FontSize};
use core_ui::prelude::*;
use core_ui::style;
use core_ui::text::Text;
use core_ui::update_element::ElementName;
use protos::spelldawn::{
    FlexAlign, FlexJustify, FlexPosition, ImageScaleMode, SpriteAddress, WhiteSpace,
};

/// Renders a full-screen image containing a text prompt and some arbitrary
/// content.
#[derive(Default)]
pub struct TilePromptPanel {
    image: SpriteAddress,
    prompt: String,
    buttons: Vec<Button>,
}

impl TilePromptPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content_name() -> ElementName {
        ElementName::new("TileImageContent")
    }

    pub fn image(mut self, image: SpriteAddress) -> Self {
        self.image = image;
        self
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    pub fn buttons(mut self, buttons: Vec<Button>) -> Self {
        self.buttons = buttons;
        self
    }
}

impl Component for TilePromptPanel {
    fn build(self) -> Option<Node> {
        Row::new("TilePromptPanel")
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::All, 0.px())
                    .background_image_scale_mode(ImageScaleMode::ScaleAndCrop)
                    .background_image(self.image),
            )
            .child(
                Column::new("Container")
                    .style(
                        Style::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Horizontal, 0.px())
                            .position(Edge::Bottom, 0.px()),
                    )
                    .child(
                        Row::new("Gradient").style(
                            Style::new()
                                .height(128.px())
                                .width(100.pct())
                                .background_image(style::sprite("Sprites/OverlayGradient")),
                        ),
                    )
                    .child(
                        Column::new(Self::content_name())
                            .style(
                                Style::new()
                                    .justify_content(FlexJustify::Center)
                                    .align_items(FlexAlign::Center)
                                    .width(100.pct())
                                    .background_color(BackgroundColor::TilePanelOverlay)
                                    .padding(Edge::All, 8.px()),
                            )
                            .child(
                                Text::new(self.prompt)
                                    .layout(Layout::new().margin(Edge::Horizontal, 64.px()))
                                    .font_size(FontSize::Headline)
                                    .white_space(WhiteSpace::Normal),
                            )
                            .child(
                                Row::new("ButtonGroup")
                                    .style(Style::new().margin(Edge::All, 8.px()))
                                    .children(self.buttons.into_iter()),
                            ),
                    ),
            )
            .build()
    }
}
