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

use element_names::ElementName;
use protos::riftcaller::{node_type, ClientAction, DraggableNode, Node, NodeType};

use crate::actions::InterfaceAction;
use crate::flexbox;
use crate::flexbox::HasNodeChildren;
use crate::prelude::*;

#[derive(Default)]
pub struct Draggable {
    render_node: Node,
    draggable: DraggableNode,
    children: Vec<Node>,
    over_target_indicator: Option<Box<dyn Fn() -> Option<Node>>>,
    enabled: bool,
}

impl Draggable {
    pub fn new(name: impl Into<String>) -> Self {
        let mut result = Self::default();
        result.render_node.name = name.into();
        result
    }

    pub fn drop_target(mut self, identifier: ElementName) -> Self {
        self.draggable.drop_target_identifiers = vec![identifier.into()];
        self
    }

    pub fn over_target_indicator(mut self, indicator: impl Fn() -> Option<Node> + 'static) -> Self {
        self.over_target_indicator = Some(Box::new(indicator));
        self
    }

    pub fn on_drop(mut self, action: Option<impl InterfaceAction + 'static>) -> Self {
        self.enabled = action.is_some();
        self.draggable.on_drop =
            action.map(|d| ClientAction { action: Some(d.as_client_action()) });
        self
    }

    /// User must drag the element through this horizontal distance in screen
    /// pixels before the UI responds. Useful to enable horizontal element
    /// dragging from a vertical scroll view.
    pub fn horizontal_drag_start_distance(mut self, offset: u32) -> Self {
        self.draggable.horizontal_drag_start_distance = Some(offset);
        self
    }

    /// If true, the original element is hidden on drag, making it appear that
    /// you are moving it directly instead of a placeholder.
    pub fn remove_original(mut self, value: bool) -> Self {
        self.draggable.remove_original = value;
        self
    }

    /// Identifiers of children of this Draggable which should be hidden in the
    /// drag indicator element.
    pub fn hide_indicator_children(mut self, children: Vec<impl Into<String>>) -> Self {
        self.draggable.hide_indicator_children = children.into_iter().map(Into::into).collect();
        self
    }

    /// Optionally, a UI element to use for the drag indicator instead of
    /// cloning this element directly.
    pub fn custom_drag_indicator(mut self, indicator: Option<Node>) -> Self {
        self.draggable.custom_drag_indicator = indicator.map(Box::new);
        self
    }

    /// Action to invoke when a gesture has been confirmed as a drag, i.e. the
    /// element has been dragged through some fixed distance.
    pub fn on_drag_detected(mut self, action: Option<impl InterfaceAction + 'static>) -> Self {
        self.enabled = action.is_some();
        self.draggable.on_drag_detected =
            action.map(|d| ClientAction { action: Some(d.as_client_action()) });
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
        self.draggable.over_target_indicator = if self.enabled {
            // Only build indicator when a drop action is actually present -- prevents
            // infinitely deep hierarchies.
            self.over_target_indicator.and_then(|indicator| indicator()).map(Box::new)
        } else {
            None
        };

        self.render_node.node_type = Some(Box::new(NodeType {
            node_type: Some(node_type::NodeType::DraggableNode(Box::new(self.draggable))),
        }));

        flexbox::build_with_children(self.render_node, self.children)
    }
}
