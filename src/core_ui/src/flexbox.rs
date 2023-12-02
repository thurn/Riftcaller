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

use std::fmt::Debug;
use std::marker::PhantomData;

use protos::riftcaller::{EventHandlers, FlexDirection, FlexStyle, Node};

use crate::actions::InterfaceAction;
use crate::component::{Component, ComponentObject};
use crate::style::Style;

/// Renders a [Flexbox] which lays out its children horizontally, from left to
/// right
pub type Row = Flexbox<RowDirection>;

/// Renders a [Flexbox] which lays out its children vertically, from top to
/// bottom
pub type Column = Flexbox<ColumnDirection>;

/// Renders a reversed (right-to-left) [Row]
pub type ReverseRow = Flexbox<ReverseRowDirection>;

/// Renders a reversed (bottom-to-top) [Column]
pub type ReverseColumn = Flexbox<ReverseColumnDirection>;

/// Marker trait to control the direction of a [Flexbox]
pub trait FlexboxDirection: Default + Debug {
    fn direction() -> FlexDirection;
}

#[derive(Debug, Default)]
pub struct RowDirection {}

impl FlexboxDirection for RowDirection {
    fn direction() -> FlexDirection {
        FlexDirection::Row
    }
}

#[derive(Debug, Default)]
pub struct ColumnDirection {}

impl FlexboxDirection for ColumnDirection {
    fn direction() -> FlexDirection {
        FlexDirection::Column
    }
}

#[derive(Debug, Default)]
pub struct ReverseRowDirection {}

impl FlexboxDirection for ReverseRowDirection {
    fn direction() -> FlexDirection {
        FlexDirection::RowReverse
    }
}

#[derive(Debug, Default)]
pub struct ReverseColumnDirection {}

impl FlexboxDirection for ReverseColumnDirection {
    fn direction() -> FlexDirection {
        FlexDirection::ColumnReverse
    }
}

/// Marker trait for any type which directly renders a [Node] and can be styled
/// by [Style].
pub trait HasRenderNode: Sized {
    fn render_node(&mut self) -> &mut Node;

    fn flex_direction(&self) -> Option<FlexDirection> {
        None
    }

    /// Name for this component. Used for debugging.
    fn name(mut self, name: impl Into<String>) -> Self {
        self.render_node().name = name.into();
        self
    }

    /// Primary [Style] used when the component is not hovered or pressed.
    fn style(mut self, style: Style) -> Self {
        self.render_node().style = Some(style.wrapped_style());
        self
    }

    /// [Style] to merge into this component's base style when it is hovered
    fn hover_style(mut self, style: Style) -> Self {
        self.render_node().hover_style = Some(style.wrapped_style());
        self
    }

    /// [Style] to merge into this component's base style when it is pressed
    fn pressed_style(mut self, style: Style) -> Self {
        self.render_node().pressed_style = Some(style.wrapped_style());
        self
    }

    /// [Style] to merge into this component's base style when it is first
    /// attached to a panel.
    fn on_attach_style(mut self, style: Style) -> Self {
        self.render_node().on_attach_style = Some(style.wrapped_style());
        self
    }

    /// Action to invoke when this component is clicked/tapped
    fn on_click(mut self, action: impl InterfaceAction + 'static) -> Self {
        self.render_node().event_handlers.get_or_insert(EventHandlers::default()).on_click =
            Some(action.build());
        self
    }

    /// Action to invoke when this component is pressed down for 500ms
    fn on_long_press(mut self, action: impl InterfaceAction + 'static) -> Self {
        self.render_node().event_handlers.get_or_insert(EventHandlers::default()).on_long_press =
            Some(action.build());
        self
    }

    /// Action to invoke when this component is pressed down
    fn on_mouse_down(mut self, action: impl InterfaceAction + 'static) -> Self {
        self.render_node().event_handlers.get_or_insert(EventHandlers::default()).on_mouse_down =
            Some(action.build());
        self
    }

    /// Action to invoke when a press on this component is released
    fn on_mouse_up(mut self, action: impl InterfaceAction + 'static) -> Self {
        self.render_node().event_handlers.get_or_insert(EventHandlers::default()).on_mouse_up =
            Some(action.build());
        self
    }

    /// Action to invoke when a text field's content changes
    fn on_field_changed(mut self, action: impl InterfaceAction + 'static) -> Self {
        self.render_node()
            .event_handlers
            .get_or_insert(EventHandlers::default())
            .on_field_changed = Some(action.build());
        self
    }
}

pub trait HasNodeChildren: HasRenderNode {
    fn get_internal_children(&mut self) -> &mut Vec<Node>;

    fn child(mut self, child: impl Component) -> Self {
        if let Some(n) = child.build() {
            self.get_internal_children().push(n);
        }
        self
    }

    fn child_node(mut self, child: Option<Node>) -> Self {
        if let Some(n) = child {
            self.get_internal_children().push(n);
        }
        self
    }

    fn child_boxed(mut self, child: Box<dyn ComponentObject>) -> Self {
        if let Some(n) = child.build_boxed() {
            self.get_internal_children().push(n);
        }
        self
    }

    fn children(mut self, children: impl Iterator<Item = impl Component>) -> Self {
        for child in children {
            if let Some(n) = child.build() {
                self.get_internal_children().push(n);
            }
        }
        self
    }

    fn child_nodes(mut self, children: impl Iterator<Item = Option<Node>>) -> Self {
        self.get_internal_children().extend(children.flatten());
        self
    }

    fn children_boxed(mut self, children: Vec<Box<dyn ComponentObject>>) -> Self {
        for child in children {
            if let Some(n) = child.build_boxed() {
                self.get_internal_children().push(n);
            }
        }
        self
    }
}

/// Primary container component for the UI system. Lays out its children
/// following flexbox spacing rules. Typically used via its [Row] or [Column]
/// aliases.
#[derive(Debug, Default)]
pub struct Flexbox<D: FlexboxDirection> {
    children: Vec<Node>,
    render_node: Node,
    phantom: PhantomData<D>,
}

impl<D: FlexboxDirection> HasRenderNode for Flexbox<D> {
    fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }

    fn flex_direction(&self) -> Option<FlexDirection> {
        Some(D::direction())
    }
}

impl<D: FlexboxDirection> HasNodeChildren for Flexbox<D> {
    fn get_internal_children(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }
}

impl<D: FlexboxDirection> Flexbox<D> {
    pub fn new(name: impl Into<String>) -> Self {
        let mut result = Self::default();
        result.render_node.name = name.into();
        result
    }
}

/// Adds the provided child components to the 'render_node' [Node] by invoking
/// their `build()` method.
pub fn build_with_children(mut render_node: Node, children: Vec<Node>) -> Option<Node> {
    render_node.children = children;
    Some(render_node)
}

impl<D: FlexboxDirection> Component for Flexbox<D> {
    fn build(mut self) -> Option<Node> {
        if let Some(d) = self.flex_direction() {
            if let Some(style) = &mut self.render_node.style {
                style.flex_direction = d.into();
            } else {
                self.render_node.style =
                    Some(Box::new(FlexStyle { flex_direction: d.into(), ..FlexStyle::default() }))
            }
        }
        build_with_children(self.render_node, self.children)
    }
}
