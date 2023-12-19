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

use protos::riftcaller::{
    node_type, Node, NodeType, ScrollBar, ScrollBarVisibility, ScrollViewNode, TouchScrollBehavior,
};

use crate::flexbox;
use crate::flexbox::HasNodeChildren;
use crate::prelude::*;

#[derive(Debug, Default)]
pub struct ScrollView {
    render_node: Node,
    children: Vec<Node>,
    scroll_node: ScrollViewNode,
}

impl ScrollView {
    pub fn new(name: impl Into<String>) -> Self {
        let mut result = Self::default();
        result.render_node.name = name.into();
        result
    }

    pub fn horizontal_scrollbar_visibility(mut self, visibility: ScrollBarVisibility) -> Self {
        self.scroll_node.set_horizontal_scroll_bar_visibility(visibility);
        self
    }

    pub fn vertical_scrollbar_visibility(mut self, visibility: ScrollBarVisibility) -> Self {
        self.scroll_node.set_vertical_scroll_bar_visibility(visibility);
        self
    }

    pub fn scroll_deceleration_rate(mut self, rate: f32) -> Self {
        self.scroll_node.scroll_deceleration_rate = Some(rate);
        self
    }

    pub fn vertical_page_size(mut self, size: f32) -> Self {
        self.scroll_node.vertical_page_size = Some(size);
        self
    }

    pub fn touch_scroll_behavior(mut self, behavior: TouchScrollBehavior) -> Self {
        self.scroll_node.set_touch_scroll_behavior(behavior);
        self
    }

    pub fn mouse_wheel_scroll_size(mut self, size: f32) -> Self {
        self.scroll_node.mouse_wheel_scroll_size = Some(size);
        self
    }

    pub fn vertical_scroll_bar_style(mut self, style: Style) -> Self {
        self.scroll_node.vertical_scroll_bar =
            Some(Box::new(ScrollBar { style: Some(style.wrapped_style()) }));
        self
    }
}

impl HasRenderNode for ScrollView {
    fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }
}

impl HasNodeChildren for ScrollView {
    fn get_internal_children(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }
}

impl Component for ScrollView {
    fn build(mut self) -> Option<Node> {
        self.render_node.node_type = Some(Box::new(NodeType {
            node_type: Some(node_type::NodeType::ScrollViewNode(Box::new(self.scroll_node))),
        }));
        flexbox::build_with_children(self.render_node, self.children)
    }
}
