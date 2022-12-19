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

use protos::spelldawn::game_command::Command;
use protos::spelldawn::toggle_panel_command::ToggleCommand;
use protos::spelldawn::{
    node_type, CardAnchorNode, EventHandlers, InterfacePanelAddress, Node, NodeType,
};

/// Simulated user interface state
#[derive(Debug, Clone, Default)]
pub struct ClientInterface {
    main_controls: Option<Node>,
    card_anchors: Vec<CardAnchorNode>,
    panels: HashMap<InterfacePanelAddress, Node>,
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
        let address = self.open_panels.last().expect("No open panel");
        self.panels.get(address).unwrap_or_else(|| panic!("Panel not found: {:?}", address))
    }

    pub fn screen_overlay(&self) -> &Node {
        self.screen_overlay.as_ref().expect("ScreenOverlayNode")
    }

    pub fn panel_count(&self) -> usize {
        self.open_panels.len()
    }

    pub fn update(&mut self, command: Command) {
        match command {
            Command::UpdateGameView(update) => {
                let controls = update.game.as_ref().expect("game").main_controls.as_ref();
                self.main_controls = controls.and_then(|c| c.node.clone());
                self.card_anchors = controls.map_or(vec![], |c| c.card_anchor_nodes.clone());
            }
            Command::UpdatePanels(panels) => {
                for panel in panels.panels {
                    self.panels.insert(panel.address.expect("address"), panel.node.expect("node"));
                }
            }
            Command::TogglePanel(toggle) => {
                self.handle_toggle(toggle.toggle_command.expect("ToggleCommand"))
            }
            Command::RenderScreenOverlay(overlay) => {
                self.screen_overlay = overlay.node;
            }
            _ => {}
        }
    }

    fn handle_toggle(&mut self, command: ToggleCommand) {
        match command {
            ToggleCommand::LoadPanel(load) => {
                let address = load.open_panel.expect("address");
                if !self.open_panels.contains(&address) {
                    self.open_panels.push(address);
                }
            }
            ToggleCommand::SetPanel(address) => {
                self.open_panels.clear();
                self.open_panels.push(address);
            }
            ToggleCommand::OpenPanel(address) => {
                if !self.open_panels.contains(&address) {
                    self.open_panels.push(address);
                }
            }
            ToggleCommand::ClosePanel(address) => {
                self.open_panels.retain(|a| *a != address);
            }
            ToggleCommand::CloseAll(_) => {
                self.open_panels.clear();
            }
            ToggleCommand::OpenBottomSheetAddress(_) => {
                todo!("Implement")
            }
            ToggleCommand::CloseBottomSheet(_) => {
                todo!("Implement")
            }
            ToggleCommand::PushBottomSheetAddress(_) => {
                todo!("Implement")
            }
            ToggleCommand::PopToBottomSheetAddress(_) => {
                todo!("Implement")
            }
        }
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
