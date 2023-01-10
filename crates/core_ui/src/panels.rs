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

use data::user_actions::UserAction;
use protos::spelldawn::client_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::toggle_panel_command::ToggleCommand;
use protos::spelldawn::{
    AddressWithLoadingState, InterfacePanel, InterfacePanelAddress, PanelTransitionOptions,
    StandardAction, TogglePanelCommand, UpdatePanelsCommand,
};

use crate::actions::{self, InterfaceAction};
use crate::prelude::*;

/// Fluent builder to help open and close panels
#[derive(Clone)]
pub struct Panels {
    open: Option<InterfacePanelAddress>,
    close: Option<InterfacePanelAddress>,
    loading: Option<InterfacePanelAddress>,
    action: Option<UserAction>,
    do_not_fetch: bool,
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
}

impl From<Panels> for Command {
    fn from(panels: Panels) -> Self {
        Command::TogglePanel(TogglePanelCommand {
            toggle_command: Some(ToggleCommand::Transition(PanelTransitionOptions {
                open: panels.open,
                close: panels.close,
                loading: panels.loading,
                do_not_fetch: panels.do_not_fetch,
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

/// Set the indicated panel as the only open panel
pub fn set(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::SetPanel(address.into())),
    })
}

/// Add the indicated panel to the end of the stack of open views if
/// it is not already present.
pub fn open(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::OpenPanel(address.into())),
    })
}

/// Add the indicated panel to the end of the stack of open views if
/// it is not already present, throwing an exception if this panel is not
/// cached. Does not attempt to fetch the panel from the server.
pub fn open_existing(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::OpenExistingPanel(address.into())),
    })
}

/// Close the 'from_address' panel and add the 'to_address' panel to the end of
/// the stack of open views if it is not already present, displaying the
/// provided component as a loading state if the panel in question is not
/// already cached.
pub fn transition(
    from_address: impl Into<InterfacePanelAddress>,
    to_address: impl Into<InterfacePanelAddress>,
    loading: impl Component + 'static,
) -> Vec<Command> {
    vec![
        close(from_address),
        Command::TogglePanel(TogglePanelCommand {
            toggle_command: Some(ToggleCommand::LoadPanel(AddressWithLoadingState {
                open_panel: Some(to_address.into()),
                loading_state: loading.build(),
            })),
        }),
    ]
}

/// Close the 'from' panel and display 'loading' 'while waiting for the 'to'
/// panel contents.
///
/// Does *not* request the address from the server, it is assumed that some
/// state mutation will cause the panel to be refreshed.
pub fn close_and_wait_for(
    from_address: impl Into<InterfacePanelAddress>,
    to_address: impl Into<InterfacePanelAddress>,
    loading: impl Component + 'static,
) -> Vec<Command> {
    vec![
        close(from_address),
        Command::TogglePanel(TogglePanelCommand {
            toggle_command: Some(ToggleCommand::WaitFor(AddressWithLoadingState {
                open_panel: Some(to_address.into()),
                loading_state: loading.build(),
            })),
        }),
    ]
}

/// Opens a new bottom sheet with the indicated panel.
///
/// Closes any existing bottom sheet.
pub fn open_bottom_sheet(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::OpenBottomSheetAddress(address.into())),
    })
}

/// Pushes the indicated panel as a new bottom sheet page.
///
/// If no bottom sheet is currently open, the behavior is identical to
/// [open_bottom_sheet].
pub fn push_bottom_sheet(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::PushBottomSheetAddress(address.into())),
    })
}

/// Removes the indicated panel from the stack of open views.
pub fn close(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::ClosePanel(address.into())),
    })
}

/// Closes all open panels
pub fn close_all() -> Command {
    Command::TogglePanel(TogglePanelCommand { toggle_command: Some(ToggleCommand::CloseAll(())) })
}

/// Closes the currently-open bottom sheet.
pub fn close_bottom_sheet() -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::CloseBottomSheet(())),
    })
}

/// Pops the currently-open bottom sheet page, displaying 'address' as the *new*
/// sheet contents.
pub fn pop_to_bottom_sheet(address: impl Into<InterfacePanelAddress>) -> Command {
    Command::TogglePanel(TogglePanelCommand {
        toggle_command: Some(ToggleCommand::PopToBottomSheetAddress(address.into())),
    })
}

/// Command to update the contents of a panel
pub fn update(panel: InterfacePanel) -> Command {
    Command::UpdatePanels(UpdatePanelsCommand { panels: vec![panel] })
}
