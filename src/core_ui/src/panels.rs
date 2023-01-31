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
use protos::spelldawn::toggle_panel_command::ToggleCommand;
use protos::spelldawn::{
    InterfacePanel, InterfacePanelAddress, PanelTransitionOptions, StandardAction,
    TogglePanelCommand, UpdatePanelsCommand,
};
use user_action_data::UserAction;

use crate::actions::{self, InterfaceAction};

/// Fluent builder to help open and close panels
#[derive(Clone)]
pub struct Panels {
    open: Option<InterfacePanelAddress>,
    close: Option<InterfacePanelAddress>,
    loading: Option<InterfacePanelAddress>,
    action: Option<UserAction>,
    do_not_fetch: bool,
    wait_to_load: bool,
}

impl Panels {
    /// Request to open the provided panel.
    pub fn open(address: impl Into<InterfacePanelAddress>) -> Self {
        Self {
            open: Some(address.into()),
            close: None,
            loading: None,
            action: None,
            do_not_fetch: false,
            wait_to_load: false,
        }
    }

    /// Request to close the indicated panel
    pub fn close(address: impl Into<InterfacePanelAddress>) -> Self {
        Self {
            open: None,
            close: Some(address.into()),
            loading: None,
            action: None,
            do_not_fetch: false,
            wait_to_load: false,
        }
    }

    /// Provides a loading state address to display while the 'open' panel is
    /// being fetched.
    ///
    /// An error will be logged if you attempt to open a panel which is not
    /// cached without a loading state, or if the loading state is itself
    /// not cached.
    pub fn loading(mut self, loading: impl Into<InterfacePanelAddress>) -> Self {
        self.loading = Some(loading.into());
        self
    }

    /// Close the indicated panel before opening the provided new panel.
    pub fn and_close(mut self, close: impl Into<InterfacePanelAddress>) -> Self {
        self.close = Some(close.into());
        self
    }

    /// Close the indicated panel before opening the provided new panel.
    pub fn and_open(mut self, open: impl Into<InterfacePanelAddress>) -> Self {
        self.open = Some(open.into());
        self
    }

    /// Send the provided action to the server when opening this panel.
    pub fn action(mut self, action: impl Into<UserAction>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// If true, do not attempt to fetch the provided 'open' panel, just wait
    /// for it to be returned.
    pub fn do_not_fetch(mut self, do_not_fetch: bool) -> Self {
        self.do_not_fetch = do_not_fetch;
        self
    }

    /// If true, displays a loading animation on the 'close' screen while
    /// fetching the 'open' screen, then transitions once it is loaded.
    pub fn wait_to_load(mut self, wait_to_load: bool) -> Self {
        self.wait_to_load = wait_to_load;
        self
    }
}

impl From<Panels> for Command {
    fn from(panels: Panels) -> Self {
        Command::TogglePanel(TogglePanelCommand {
            toggle_command: Some(ToggleCommand::Transition(PanelTransitionOptions {
                open: panels.open,
                close: panels.close,
                loading: panels.loading,
                do_not_fetch: panels.do_not_fetch,
                wait_to_load: panels.wait_to_load,
            })),
        })
    }
}

impl InterfaceAction for Panels {
    fn as_client_action(&self) -> Action {
        let clone: Panels = self.clone();
        Action::StandardAction(StandardAction {
            payload: clone.action.map_or_else(Vec::new, actions::payload),
            update: Some(actions::command_list(vec![clone.into()])),
            request_fields: HashMap::new(),
        })
    }
}

/// Command to update the contents of a panel
pub fn update(panel: InterfacePanel) -> Command {
    Command::UpdatePanels(UpdatePanelsCommand { panels: vec![panel] })
}
