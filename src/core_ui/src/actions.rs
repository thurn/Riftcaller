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

use std::collections::HashMap;
use std::fmt::Debug;

use adventure_data::adventure_action::AdventureAction;
use core_data::game_primitives::ResponseContext;
use game_data::game_actions::{GameAction, GameStateAction};
use protos::riftcaller::client_action::Action;
use protos::riftcaller::game_command::Command;
use protos::riftcaller::{ClientAction, CommandList, GameCommand, LoggingMetadata, StandardAction};
use serde_json::ser;
use user_action_data::{DebugAction, UserAction};

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

impl InterfaceAction for GameStateAction {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction {
            payload: payload(UserAction::GameAction(GameAction::GameStateAction(*self))),
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
            update: Some(command_list(None, vec![self.clone()])),
            request_fields: HashMap::new(),
        })
    }
}

impl InterfaceAction for Vec<Command> {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction {
            payload: vec![],
            update: Some(command_list(None, self.clone())),
            request_fields: HashMap::new(),
        })
    }
}

impl InterfaceAction for Vec<Option<Command>> {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(StandardAction {
            payload: vec![],
            update: Some(command_list(None, self.iter().flatten().cloned().collect())),
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
        update: Some(command_list(None, update)),
        request_fields: HashMap::new(),
    })
}

pub fn with_request_fields(action: impl Into<UserAction>, fields: Vec<String>) -> Action {
    Action::StandardAction(StandardAction {
        payload: payload(action.into()),
        update: None,
        request_fields: fields.into_iter().map(|name| (name, String::new())).collect(),
    })
}

pub fn payload(action: UserAction) -> Vec<u8> {
    ser::to_vec(&action).expect("Serialization failed")
}

pub fn command_list(context: Option<ResponseContext>, commands: Vec<Command>) -> CommandList {
    CommandList {
        logging_metadata: logging_metadata(context),
        commands: commands.into_iter().map(|c| GameCommand { command: Some(c) }).collect(),
        metadata: None,
    }
}

fn logging_metadata(context: Option<ResponseContext>) -> Vec<LoggingMetadata> {
    match context {
        Some(ResponseContext::Adventure(adventure_id)) => {
            vec![LoggingMetadata {
                key: "adventure_id".to_string(),
                value: adventure_id.to_string(),
            }]
        }
        Some(ResponseContext::Game(game_id)) => {
            vec![LoggingMetadata { key: "game_id".to_string(), value: game_id.to_string() }]
        }
        Some(ResponseContext::LeaveAdventure) => {
            vec![LoggingMetadata { key: "adventure_id".to_string(), value: String::new() }]
        }
        Some(ResponseContext::LeaveGame) => {
            vec![LoggingMetadata { key: "game_id".to_string(), value: String::new() }]
        }
        _ => vec![],
    }
}
