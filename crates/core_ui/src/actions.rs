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
use std::fmt::Debug;

use data::adventure_action::AdventureAction;
use data::game_actions::{GameAction, PromptAction};
use data::user_actions::{DebugAction, UserAction};
use protos::spelldawn::client_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{ClientAction, CommandList, GameCommand, StandardAction};
use serde_json::ser;

/// Represents an action that can be performed in the user interface. Initiating
/// a server request and performing an immediate client update are both
/// supported forms of action.
pub trait InterfaceAction {
    fn as_client_action(&self) -> Action;

    /// Converts an [InterfaceAction] into a [ClientAction].
    fn build(&self) -> ClientAction {
        ClientAction { action: Some(self.as_client_action()) }
    }
}

impl InterfaceAction for Action {
    fn as_client_action(&self) -> Action {
        self.clone()
    }
}

impl InterfaceAction for StandardAction {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(self.clone())
    }
}

/// Marker struct for when no action is desired.
#[derive(Debug)]
pub struct NoAction {}

impl InterfaceAction for NoAction {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction::default())
    }
}

impl InterfaceAction for UserAction {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction {
            payload: payload(*self),
            update: None,
            request_fields: HashMap::new(),
        })
    }
}

impl InterfaceAction for DebugAction {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction {
            payload: payload(UserAction::Debug(*self)),
            update: None,
            request_fields: HashMap::new(),
        })
    }
}

impl InterfaceAction for GameAction {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction {
            payload: payload(UserAction::GameAction(*self)),
            update: None,
            request_fields: HashMap::new(),
        })
    }
}

impl InterfaceAction for PromptAction {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction {
            payload: payload(UserAction::GameAction(GameAction::PromptAction(*self))),
            update: None,
            request_fields: HashMap::new(),
        })
    }
}

impl InterfaceAction for AdventureAction {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction {
            payload: payload(UserAction::AdventureAction(*self)),
            update: None,
            request_fields: HashMap::new(),
        })
    }
}

impl InterfaceAction for Command {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction {
            payload: vec![],
            update: Some(command_list(vec![self.clone()])),
            request_fields: HashMap::new(),
        })
    }
}

impl InterfaceAction for Vec<Command> {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction {
            payload: vec![],
            update: Some(command_list(self.clone())),
            request_fields: HashMap::new(),
        })
    }
}

/// Returns a StandardAction to apply an optimistic UI update via a series on
/// commands on click, followed by a regular game action specified as an
/// InterfaceAction.
///
/// The provided InterfaceAction must map to a StandardAction, or else this
/// function will panic.
pub fn with_optimistic_update(
    update: Vec<Command>,
    action: impl InterfaceAction + 'static,
) -> Action {
    let payload = match action.as_client_action() {
        Action::StandardAction(standard_action) => standard_action.payload,
        _ => panic!("Expected StandardAction"),
    };

    Action::StandardAction(StandardAction {
        payload,
        update: Some(command_list(update)),
        request_fields: HashMap::new(),
    })
}

pub fn payload(action: UserAction) -> Vec<u8> {
    ser::to_vec(&action).expect("Serialization failed")
}

pub fn command_list(commands: Vec<Command>) -> CommandList {
    CommandList {
        commands: commands.into_iter().map(|c| GameCommand { command: Some(c) }).collect(),
    }
}
