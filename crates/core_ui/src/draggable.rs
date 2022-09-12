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

use protos::spelldawn::{node_type, ClientAction, DraggableNode, Node, NodeType};

use crate::actions::InterfaceAction;
use crate::flexbox;
use crate::flexbox::HasNodeChildren;
use crate::prelude::*;

#[derive(Default)]
pub struct Draggable {
    render_node: Node,
    children: Vec<Node>,
    drop_targets: Vec<String>,
    over_target_indicator: Option<Box<dyn Fn() -> Option<Node>>>,
    on_drop: Option<Box<dyn InterfaceAction>>,
}

impl Draggable {
    pub fn new(name: impl Into<String>) -> Self {
        let mut result = Self::default();
        result.render_node.name = name.into();
        result
    }

    pub fn drop_targets(mut self, identifiers: Vec<impl Into<String>>) -> Self {
        self.drop_targets = identifiers.into_iter().map(Into::into).collect();
        self
    }

    pub fn over_target_indicator(mut self, indicator: impl Fn() -> Option<Node> + 'static) -> Self {
        self.over_target_indicator = Some(Box::new(indicator));
        self
    }

    pub fn on_drop(mut self, action: Option<impl InterfaceAction + 'static>) -> Self {
        if let Some(a) = action {
            self.on_drop = Some(Box::new(a));
        }
        self
    }
}

impl HasRenderNode for Draggable {
    fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }
}

impl HasNodeChildren for Draggable {
    fn get_internal_children(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }
}

impl Component for Draggable {
    fn build(mut self) -> Option<Node> {
        dbg!("Building draggable");
        self.render_node.node_type = Some(Box::new(NodeType {
            node_type: Some(node_type::NodeType::DraggableNode(Box::new(DraggableNode {
                drop_target_identifiers: self.drop_targets,
                over_target_indicator: if self.on_drop.is_some() {
                    // Only build indicator when a drop action is actually present -- prevents
                    // infinitely deep hierarchies.
                    self.over_target_indicator.and_then(|indicator| indicator()).map(Box::new)
                } else {
                    None
                },
                on_drop: self.on_drop.map(|d| ClientAction { action: d.as_client_action() }),
            }))),
        }));

        flexbox::build_with_children(self.render_node, self.children)
    }
}
