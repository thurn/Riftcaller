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

use std::collections::HashMap;

use protos::spelldawn::client_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::node_background::BackgroundAddress;
use protos::spelldawn::studio_display::Display;
use protos::spelldawn::toggle_panel_command::ToggleCommand;
use protos::spelldawn::{
    node_type, CardAnchorNode, CardView, ClientAction, DraggableNode, EventHandlers,
    FetchPanelAction, InterfacePanel, InterfacePanelAddress, Node, NodeType, StudioDisplay,
};

/// Simulated user interface state
#[derive(Debug, Clone, Default)]
pub struct ClientInterface {
    main_controls: Option<Node>,
    card_anchors: Vec<CardAnchorNode>,
    panels: HashMap<InterfacePanelAddress, InterfacePanel>,
    open_panels: Vec<InterfacePanelAddress>,
    screen_overlay: Option<Node>,
}

impl ClientInterface {
    pub fn main_controls_option(&self) -> Option<Node> {
        self.main_controls.clone()
    }

    pub fn main_controls(&self) -> &Node {
        self.main_controls.as_ref().expect("MainControls Node")
    }

    pub fn card_anchors(&self) -> &Vec<CardAnchorNode> {
        &self.card_anchors
    }

    pub fn controls(&self) -> Vec<&Node> {
        let mut result =
            vec![self.main_controls.as_ref()].into_iter().flatten().collect::<Vec<_>>();
        result.extend(self.card_anchor_nodes());
        result
    }

    pub fn card_anchor_nodes(&self) -> Vec<&Node> {
        self.card_anchors().iter().filter_map(|node| node.node.as_ref()).collect()
    }

    /// Returns the contents of the topmost currently-open panel
    pub fn top_panel(&self) -> &Node {
        self.top_panel_option().expect("No panel found")
    }

    pub fn top_panel_option(&self) -> Option<&Node> {
        if let Some(address) = self.open_panels.last() {
            self.panels
                .get(address)
                .unwrap_or_else(|| panic!("Panel not found: {:?}", address.debug_string))
                .node
                .as_ref()
        } else {
            None
        }
    }

    pub fn screen_overlay(&self) -> &Node {
        self.screen_overlay_option().expect("Screen overlay not found")
    }

    pub fn screen_overlay_option(&self) -> Option<&Node> {
        if let Some(overlay) = self
            .open_panels
            .iter()
            .rev()
            .find_map(|address| self.panels.get(address)?.screen_overlay.as_ref())
        {
            Some(overlay)
        } else {
            self.screen_overlay.as_ref()
        }
    }

    pub fn all_active_nodes(&self) -> Vec<&Node> {
        let mut result =
            vec![self.main_controls.as_ref()].into_iter().flatten().collect::<Vec<_>>();
        result.extend(self.card_anchor_nodes());
        if let Some(panel) = self.top_panel_option() {
            result.push(panel);
        }
        if let Some(overlay) = self.screen_overlay_option() {
            result.push(overlay);
        }
        result
    }

    pub fn panel_count(&self) -> usize {
        self.open_panels.len()
    }

    pub fn open_panels(&self) -> Vec<InterfacePanelAddress> {
        self.open_panels.clone()
    }

    pub fn update(&mut self, command: Command) -> Vec<ClientAction> {
        let mut actions = vec![];
        match command {
            Command::UpdateGameView(update) => {
                let controls = update.game.as_ref().expect("game").main_controls.as_ref();
                self.main_controls = controls.and_then(|c| c.node.clone());
                self.card_anchors = controls.map_or(vec![], |c| c.card_anchor_nodes.clone());
            }
            Command::UpdatePanels(panels) => {
                for panel in panels.panels {
                    let address = panel.address.clone().expect("address");
                    self.panels.insert(address, panel);
                }
            }
            Command::TogglePanel(toggle) => {
                actions.extend(self.handle_toggle(toggle.toggle_command.expect("ToggleCommand")));
            }
            Command::RenderScreenOverlay(overlay) => {
                self.screen_overlay = overlay.node;
            }
            _ => {}
        }

        actions
    }

    fn handle_toggle(&mut self, command: ToggleCommand) -> Vec<ClientAction> {
        match command {
            ToggleCommand::Transition(transition) => {
                if let Some(open) = transition.open {
                    self.open_panels.push(open);
                }

                if let Some(close) = transition.close {
                    self.open_panels.retain(|a| *a != close);
                }
            }
            _ => {
                todo!("Implement")
            }
        }

        self.open_panels
            .iter()
            .filter(|p| !self.panels.contains_key(p))
            .map(|p| ClientAction {
                action: Some(Action::FetchPanel(FetchPanelAction {
                    panel_address: Some(p.clone()),
                })),
            })
            .collect()
    }
}

/// Searches for a child element of `node` which contains 'name' in its element
/// name.
pub fn find_element_name(node: &Node, name: impl Into<String>) -> Option<&Node> {
    let n = name.into();
    if node.name.contains(&n) {
        Some(node)
    } else {
        for child in &node.children {
            if let Some(c) = find_element_name(child, n.clone()) {
                return Some(c);
            }
        }

        None
    }
}

/// Panics if no child element of `node` contains the substring `name` in its
/// element name.
pub fn assert_has_element_name(node: &Node, name: impl Into<String>) {
    let n = name.into();
    if find_element_name(node, n.clone()).is_none() {
        panic!("Element '{n}' not found!");
    }
}

/// Finds a [DraggableNode] which is a child of this Node, if any
pub fn find_draggable(node: &Node) -> Option<&DraggableNode> {
    if let Some(t) = &node.node_type {
        if let node_type::NodeType::DraggableNode(d) = t.node_type.as_ref().expect("node_type") {
            return Some(d);
        }
    }

    for child in &node.children {
        if let Some(c) = find_draggable(child) {
            return Some(c);
        }
    }

    None
}

/// Finds a [StudioDisplay] which is the background of a Node, if any
pub fn find_studio_display(node: &Node) -> Option<&StudioDisplay> {
    if let Some(display) = (|| {
        let background =
            node.style.as_ref()?.background_image.as_ref()?.background_address.as_ref();
        if let Some(BackgroundAddress::StudioDisplay(d)) = background {
            Some(d)
        } else {
            None
        }
    })() {
        return Some(display.as_ref());
    }

    for child in &node.children {
        if let Some(c) = find_studio_display(child) {
            return Some(c);
        }
    }

    None
}

/// Finds a [CardView] embedded within a [StudioDisplay] in this Node, if any.
pub fn find_card_view(node: &Node) -> Option<&CardView> {
    if let Some(Display::Card(c)) = find_studio_display(node).as_ref()?.display.as_ref() {
        Some(c.card.as_ref()?)
    } else {
        None
    }
}

pub trait HasText {
    /// Returns true if there are any text nodes contained within this tree
    /// which contain the provided string.
    fn has_text(&self, text: impl Into<String>) -> bool;

    /// Populates `path` with the series of nodes leading to the node which
    /// contains the provided text. Leaves `path` unchanged if no node
    /// containing this text is found.
    fn find_text(&self, path: &mut Vec<Node>, text: impl Into<String>);

    /// Finds the path to the provided `text` via [Self::find_text] and then
    /// searches up the path for registered [EventHandlers].
    fn find_handlers(&self, text: impl Into<String>) -> Option<EventHandlers>;

    /// Returns all text contained within this tree
    fn get_text(&self) -> Vec<String>;

    /// Returns all text elements within this node concatenated together
    fn all_text(&self) -> String {
        self.get_text().join("")
    }
}

impl HasText for Node {
    fn has_text(&self, text: impl Into<String>) -> bool {
        let string = text.into();
        if let Some(NodeType { node_type: Some(node_type::NodeType::Text(s)) }) =
            self.node_type.as_deref()
        {
            if s.label.contains(string.as_str()) {
                return true;
            }
        }

        for child in &self.children {
            if child.has_text(string.as_str()) {
                return true;
            }
        }

        false
    }

    fn find_text(&self, path: &mut Vec<Node>, text: impl Into<String>) {
        let string = text.into();
        if self.has_text(string.as_str()) {
            path.push(self.clone());
        }

        for child in &self.children {
            child.find_text(path, string.as_str());
        }
    }

    fn find_handlers(&self, text: impl Into<String>) -> Option<EventHandlers> {
        let mut nodes = vec![];
        self.find_text(&mut nodes, text);
        nodes.reverse();
        nodes.iter().find_map(|node| node.event_handlers.clone())
    }

    fn get_text(&self) -> Vec<String> {
        let mut result = vec![];
        if let Some(NodeType { node_type: Some(node_type::NodeType::Text(s)) }) =
            self.node_type.as_deref()
        {
            result.push(s.label.clone())
        }

        for child in &self.children {
            result.extend(child.get_text());
        }

        result
    }
}

impl HasText for Vec<&Node> {
    fn has_text(&self, text: impl Into<String>) -> bool {
        let string = text.into();
        for node in self {
            if node.has_text(string.as_str()) {
                return true;
            }
        }
        false
    }

    fn find_text(&self, path: &mut Vec<Node>, text: impl Into<String>) {
        let string = text.into();
        for node in self {
            if node.has_text(string.as_str()) {
                return node.find_text(path, string.as_str());
            }
        }
    }

    fn find_handlers(&self, text: impl Into<String>) -> Option<EventHandlers> {
        let string = text.into();
        for node in self {
            if let Some(handlers) = node.find_handlers(string.as_str()) {
                return Some(handlers);
            }
        }
        None
    }

    fn get_text(&self) -> Vec<String> {
        let mut result = vec![];
        for node in self {
            result.extend(node.get_text());
        }
        result
    }
}
