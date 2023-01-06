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

use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition, TextAlign};

use crate::actions::{InterfaceAction, NoAction};
use crate::design::{Font, FontColor, FontSize};
use crate::prelude::*;
use crate::style;
use crate::style::WidthMode;
use crate::text::Text;
use crate::update_element::ElementName;

#[derive(Debug, Clone, Copy)]
pub enum ButtonType {
    /// Brightly-colored button, main call to action
    Primary,
    /// Less colorful button, deemphasized action
    Secondary,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonTextSize {
    Default,
    Multiline,
}

/// Implements a standard clickable button
pub struct Button {
    label: String,
    name: Option<String>,
    layout: Layout,
    button_type: ButtonType,
    action: Box<dyn InterfaceAction>,
    two_lines: bool,
    width_mode: WidthMode,
    disabled: bool,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            name: None,
            layout: Layout::default(),
            button_type: ButtonType::Primary,
            action: Box::new(NoAction {}),
            two_lines: false,
            width_mode: WidthMode::Constrained,
            disabled: false,
        }
    }

    pub fn name(mut self, name: &ElementName) -> Self {
        self.name = Some(name.clone().into());
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn button_type(mut self, button_type: ButtonType) -> Self {
        self.button_type = button_type;
        self
    }

    pub fn action(mut self, action: impl InterfaceAction + 'static) -> Self {
        self.action = Box::new(action);
        self
    }

    pub fn two_lines(mut self, is_two_lines: bool) -> Self {
        self.two_lines = is_two_lines;
        self
    }

    pub fn width_mode(mut self, width_mode: WidthMode) -> Self {
        self.width_mode = width_mode;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Component for Button {
    fn build(self) -> Option<Node> {
        let background = style::sprite(match self.button_type {
            ButtonType::Primary => {
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Orange"
            }
            ButtonType::Secondary => {
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Gray"
            }
        });

        Row::new(self.name.unwrap_or_else(|| format!("{} Button", self.label)))
            .style(
                self.layout
                    .to_style()
                    .height(88.px())
                    .min_width(132.px())
                    .justify_content(FlexJustify::Center)
                    .align_items(FlexAlign::Center)
                    .flex_shrink(0.0)
                    .flex_grow(if self.width_mode == WidthMode::Constrained { 0.0 } else { 1.0 })
                    .background_image(background)
                    .image_slice(Edge::Horizontal, 16.px()),
            )
            .on_click(self.action.as_client_action())
            .child(
                Text::new(self.label)
                    .font_size(if self.two_lines {
                        FontSize::ButtonLabelTwoLines
                    } else {
                        FontSize::ButtonLabel
                    })
                    .color(FontColor::ButtonLabel)
                    .font(Font::ButtonLabel)
                    .text_align(TextAlign::MiddleCenter)
                    .layout(
                        Layout::new().margin(
                            Edge::Horizontal,
                            if self.two_lines { 32.px() } else { 16.px() },
                        ),
                    ),
            )
            .build()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IconButtonType {
    Secondary,
    SecondaryLarge,
    Destructive,
    DestructiveLarge,
    NavBlue,
    NavBrown,
}

pub struct IconButton {
    icon: String,
    name: String,
    layout: Layout,
    button_type: IconButtonType,
    action: Box<dyn InterfaceAction>,
    show_frame: bool,
}

impl IconButton {
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            name: "IconButton".to_string(),
            layout: Layout::default(),
            button_type: IconButtonType::Secondary,
            action: Box::new(NoAction {}),
            show_frame: false,
        }
    }

    pub fn name(mut self, name: &ElementName) -> Self {
        self.name = name.clone().into();
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn button_type(mut self, button_type: IconButtonType) -> Self {
        self.button_type = button_type;
        self
    }

    pub fn action(mut self, action: impl InterfaceAction + 'static) -> Self {
        self.action = Box::new(action);
        self
    }

    pub fn show_frame(mut self, show_frame: bool) -> Self {
        self.show_frame = show_frame;
        self
    }
}

impl Component for IconButton {
    fn build(self) -> Option<Node> {
        let frame = style::sprite(
            "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Square/EPIC_silver_fr_s",
        );

        let background = style::sprite(match self.button_type {
            IconButtonType::Secondary => {
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Square/Button_GRAY_s"
            }
            IconButtonType::SecondaryLarge => {
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Gray"
            }
            IconButtonType::Destructive => {
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Square/Button_RED_s"
            }
            IconButtonType::DestructiveLarge => {
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Red"
            }
            IconButtonType::NavBlue => "Sprites/Circle1",
            IconButtonType::NavBrown => "Sprites/Circle2",
        });

        let (background_size, position_offset) = match self.button_type {
            IconButtonType::Secondary | IconButtonType::Destructive => (56, 16),
            IconButtonType::SecondaryLarge
            | IconButtonType::DestructiveLarge
            | IconButtonType::NavBlue
            | IconButtonType::NavBrown => (88, 0),
        };

        Row::new(self.name)
            .style(
                self.layout
                    .to_style()
                    .height(88.px())
                    .width(88.px())
                    .justify_content(FlexJustify::Center)
                    .align_items(FlexAlign::Center)
                    .flex_shrink(0.0),
            )
            .on_click(self.action.as_client_action())
            .child(if self.show_frame {
                Some(
                    Row::new("Frame").style(
                        Style::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::All, 6.px())
                            .height(76.px())
                            .width(76.px())
                            .background_image(frame),
                    ),
                )
            } else {
                None
            })
            .child(
                Row::new("Background").style(
                    Style::new()
                        .position_type(FlexPosition::Absolute)
                        .position(Edge::All, position_offset.px())
                        .height(background_size.px())
                        .width(background_size.px())
                        .background_image(background),
                ),
            )
            .child(
                Text::new(self.icon)
                    .font_size(FontSize::ButtonIcon)
                    .color(FontColor::ButtonLabel)
                    .font(Font::ButtonLabel)
                    .text_align(TextAlign::MiddleCenter),
            )
            .build()
    }
}
