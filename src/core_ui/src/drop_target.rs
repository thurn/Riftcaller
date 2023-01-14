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

use protos::spelldawn::{node_type, DropTargetNode, Node, NodeType};

use crate::flexbox;
use crate::flexbox::HasNodeChildren;
use crate::prelude::*;

#[derive(Debug, Default)]
pub struct DropTarget {
    render_node: Node,
    children: Vec<Node>,
}

impl DropTarget {
    pub fn new(name: impl Into<String>) -> Self {
        let mut result = Self::default();
        result.render_node.name = name.into();
        result
    }
}

impl HasRenderNode for DropTarget {
    fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }
}

impl HasNodeChildren for DropTarget {
    fn get_internal_children(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }
}

impl Component for DropTarget {
    fn build(mut self) -> Option<Node> {
        self.render_node.node_type = Some(Box::new(NodeType {
            node_type: Some(node_type::NodeType::DropTargetNode(DropTargetNode::default())),
        }));

        flexbox::build_with_children(self.render_node, self.children)
    }
}
