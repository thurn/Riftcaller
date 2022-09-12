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

use protos::spelldawn::{node_type, Node, NodeType, ScrollBarVisibility, ScrollViewNode};

use crate::flexbox;
use crate::flexbox::HasNodeChildren;
use crate::prelude::*;

#[derive(Debug)]
pub struct ScrollView {
    render_node: Node,
    children: Vec<Node>,
}

impl ScrollView {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            render_node: Node {
                name: name.into(),
                node_type: Some(Box::new(NodeType {
                    node_type: Some(node_type::NodeType::ScrollViewNode(ScrollViewNode::default())),
                })),
                ..Node::default()
            },
            children: vec![],
        }
    }

    pub fn horizontal_scrollbar_visibility(mut self, visibility: ScrollBarVisibility) -> Self {
        self.internal_node().unwrap().horizontal_scroll_bar_visibility = visibility.into();
        self
    }

    pub fn vertical_scrollbar_visibility(mut self, visibility: ScrollBarVisibility) -> Self {
        self.internal_node().unwrap().vertical_scroll_bar_visibility = visibility.into();
        self
    }

    fn internal_node(&mut self) -> Option<&mut ScrollViewNode> {
        if let Some(node_type::NodeType::ScrollViewNode(n)) =
            self.render_node.node_type.as_mut()?.node_type.as_mut()
        {
            Some(n)
        } else {
            None
        }
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
    fn build(self) -> Option<Node> {
        flexbox::build_with_children(self.render_node, self.children)
    }
}
