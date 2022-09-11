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
use crate::flexbox::HasNodeChildren;
use crate::prelude::*;
use crate::rendering;

#[derive(Debug, Default)]
pub struct Draggable {
    render_node: Node,
    children: Vec<Box<dyn Component>>,
    drop_targets: Vec<String>,
    over_target_indicator: Option<Box<dyn Component>>,
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

    pub fn over_target_indicator(mut self, indicator: impl Component + 'static) -> Self {
        self.over_target_indicator = Some(Box::new(indicator));
        self
    }

    pub fn on_drop(mut self, action: impl InterfaceAction + 'static) -> Self {
        self.on_drop = Some(Box::new(action));
        self
    }
}

impl HasRenderNode for Draggable {
    fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }
}

impl HasNodeChildren for Draggable {
    fn get_internal_children(&mut self) -> &mut Vec<Box<dyn Component>> {
        &mut self.children
    }
}

impl Component for Draggable {
    fn build(mut self) -> RenderResult {
        self.render_node.node_type = Some(Box::new(NodeType {
            node_type: Some(node_type::NodeType::DraggableNode(Box::new(DraggableNode {
                drop_target_identifiers: self.drop_targets,
                over_target_indicator: self
                    .over_target_indicator
                    .and_then(|c| rendering::component_boxed(c).map(Box::new)),
                on_drop: self.on_drop.map(|d| ClientAction { action: d.as_client_action() }),
            }))),
        }));
        RenderResult::Container(Box::new(self.render_node), self.children)
    }
}
