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

use protos::spelldawn::{node_type, Node, NodeType, SliderNode};

use crate::design::{FontColor, FontSize};
use crate::prelude::*;

/// Displays a slider which lets you drag to pick a floating point value within
/// a given range.
#[derive(Debug, Default)]
pub struct Slider {
    render_node: Node,
    slider_node: SliderNode,
    layout: Layout,
}

impl Slider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.slider_node.label = label.into();
        self
    }

    pub fn preference_key(mut self, key: impl Into<String>) -> Self {
        self.slider_node.preference_key = key.into();
        self
    }

    pub fn low_value(mut self, value: f32) -> Self {
        self.slider_node.low_value = value;
        self
    }

    pub fn high_value(mut self, value: f32) -> Self {
        self.slider_node.high_value = value;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
}

impl Component for Slider {
    fn build(mut self) -> Option<Node> {
        self.slider_node.label_style = Some(
            Style::new()
                .margin(Edge::Right, 32.px())
                .margin(Edge::Top, 0.px())
                .margin(Edge::Bottom, 2.px())
                .margin(Edge::Left, 2.px())
                .font_size(FontSize::Body)
                .color(FontColor::PrimaryText)
                .wrapped_style(),
        );
        self.render_node.node_type = Some(Box::new(NodeType {
            node_type: Some(node_type::NodeType::SliderNode(self.slider_node)),
        }));
        self.render_node.style = Some(self.layout.to_style().wrapped_style());
        Some(self.render_node)
    }
}
