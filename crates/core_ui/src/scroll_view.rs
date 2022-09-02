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

use protos::spelldawn::{node_type, Node, NodeType, ScrollViewNode};

use crate::flexbox::HasNodeChildren;
use crate::prelude::*;

#[derive(Debug)]
pub struct ScrollView {
    render_node: Node,
    children: Vec<Box<dyn Component>>,
}

impl ScrollView {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            render_node: Node {
                name: name.into(),
                node_type: Some(NodeType {
                    node_type: Some(node_type::NodeType::ScrollViewNode(ScrollViewNode::default())),
                }),
                ..Node::default()
            },
            children: vec![],
        }
    }
}

impl HasRenderNode for ScrollView {
    fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }
}

impl HasNodeChildren for ScrollView {
    fn get_internal_children(&mut self) -> &mut Vec<Box<dyn Component>> {
        &mut self.children
    }
}

impl Component for ScrollView {
    fn build(self) -> RenderResult {
        RenderResult::Container(Box::new(self.render_node), self.children)
    }
}
