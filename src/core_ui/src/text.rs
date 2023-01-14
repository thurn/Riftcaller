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

use protos::spelldawn::{
    node_type, Dimension, FlexColor, FlexOverflow, FontStyle, Node, NodeType, TextAlign,
    TextOverflow, TextShadow, WhiteSpace,
};

use crate::design::{Font, FontColor, FontSize, BLACK};
use crate::prelude::*;
use crate::style::{Pixels, WidthMode};

/// Standard design-system-aware text-rendering component
#[derive(Debug)]
pub struct Text {
    text: String,
    size: Dimension,
    color: FlexColor,
    font: Font,
    layout: Layout,
    font_style: FontStyle,
    text_align: TextAlign,
    white_space: WhiteSpace,
    width_mode: WidthMode,
    outline_color: FlexColor,
    outline_width: Pixels,
    letter_spacing: Pixels,
    remove_padding: bool,
    shadow: Option<TextShadow>,
    overflow: TextOverflow,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: FontColor::PrimaryText.into(),
            size: FontSize::Body.into(),
            font: Font::PrimaryText,
            layout: Layout::default(),
            font_style: FontStyle::Unspecified,
            text_align: TextAlign::MiddleCenter,
            white_space: WhiteSpace::Unspecified,
            width_mode: WidthMode::Constrained,
            outline_color: BLACK,
            outline_width: 0.px(),
            letter_spacing: 0.px(),
            remove_padding: false,
            shadow: None,
            overflow: TextOverflow::Ellipsis,
        }
    }

    pub fn font_size(mut self, font_size: FontSize) -> Self {
        self.size = font_size.into();
        self
    }

    pub fn raw_font_size(mut self, font_size: Dimension) -> Self {
        self.size = font_size;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn color(mut self, color: FontColor) -> Self {
        self.color = color.into();
        self
    }

    pub fn raw_color(mut self, color: FlexColor) -> Self {
        self.color = color;
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn font_style(mut self, font_style: FontStyle) -> Self {
        self.font_style = font_style;
        self
    }

    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }

    pub fn white_space(mut self, white_space: WhiteSpace) -> Self {
        self.white_space = white_space;
        self
    }

    pub fn width_mode(mut self, width_mode: WidthMode) -> Self {
        self.width_mode = width_mode;
        self
    }

    pub fn outline_color(mut self, color: impl Into<FlexColor>) -> Self {
        self.outline_color = color.into();
        self
    }

    pub fn outline_width(mut self, width: impl Into<Pixels>) -> Self {
        self.outline_width = width.into();
        self
    }

    pub fn letter_spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.letter_spacing = spacing.into();
        self
    }

    /// Should Unity's default font padding be removed?
    pub fn remove_padding(mut self, remove_padding: bool) -> Self {
        self.remove_padding = remove_padding;
        self
    }

    pub fn shadow(mut self, shadow: TextShadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    pub fn text_overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self
    }
}

impl Component for Text {
    fn build(self) -> Option<Node> {
        let mut style = self.layout.to_style();
        if self.remove_padding {
            style = style.padding(Edge::All, 0.px());
        }

        TextNode::new(self.text)
            .style(
                style
                    .font_size(self.size)
                    .color(self.color)
                    .font(self.font)
                    .font_style(self.font_style)
                    .text_align(self.text_align)
                    .letter_spacing(self.letter_spacing)
                    .white_space(self.white_space)
                    .flex_grow(if self.width_mode == WidthMode::Constrained { 0.0 } else { 1.0 })
                    .text_overflow(self.overflow)
                    .overflow(FlexOverflow::Hidden)
                    .text_outline_color(self.outline_color)
                    .text_outline_width(self.outline_width)
                    .text_shadow(self.shadow),
            )
            .build()
    }
}

/// Low level design-system-agnostic text-rendering component
#[derive(Debug, Default)]
pub struct TextNode {
    render_node: Node,
}

impl TextNode {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            render_node: Node {
                node_type: Some(Box::new(NodeType {
                    node_type: Some(node_type::NodeType::Text(protos::spelldawn::Text {
                        label: text.into(),
                    })),
                })),
                ..Node::default()
            },
        }
    }
}

impl HasRenderNode for TextNode {
    fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }
}

impl Component for TextNode {
    fn build(self) -> Option<Node> {
        Some(self.render_node)
    }
}
