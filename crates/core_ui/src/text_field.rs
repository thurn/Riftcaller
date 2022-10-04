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

use protos::spelldawn::{node_type, NodeType, TextFieldNode};

use crate::design::FontSize;
use crate::flexbox;
use crate::flexbox::HasNodeChildren;
use crate::prelude::*;

#[derive(Debug, Default)]
pub struct TextField {
    render_node: Node,
    children: Vec<Node>,
    field_node: TextFieldNode,
    layout: Layout,
}

impl TextField {
    /// Globally unique identifier for this text field, used to avoid
    /// overwriting user input. Cannot be the empty string.
    //
    /// An initial value will only be set once on the TextField for a given
    /// identifier.
    pub fn new(global_identifier: impl Into<String>) -> Self {
        let mut result = Self::default();
        let identifier = global_identifier.into();
        assert_ne!(identifier, String::new(), "Identifier cannot be empty");
        result.render_node.name = identifier.clone();
        result.field_node.global_identifier = identifier;
        result
    }

    /// Text to initially display within the text field.
    pub fn initial_text(mut self, text: impl Into<String>) -> Self {
        self.field_node.initial_text = text.into();
        self
    }

    // Maximum number of characters for the field.
    pub fn max_characters(mut self, max: u32) -> Self {
        self.field_node.max_length = max;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
}

impl HasRenderNode for TextField {
    fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }
}

impl HasNodeChildren for TextField {
    fn get_internal_children(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }
}

impl Component for TextField {
    fn build(mut self) -> Option<Node> {
        self.render_node.node_type = Some(Box::new(NodeType {
            node_type: Some(node_type::NodeType::TextFieldNode(self.field_node)),
        }));
        self.render_node.style =
            Some(self.layout.to_style().flex_grow(1.0).font_size(FontSize::Body).wrapped_style());
        flexbox::build_with_children(self.render_node, self.children)
    }
}
