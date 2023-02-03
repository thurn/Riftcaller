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

use protos::spelldawn::client_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::StandardAction;
use user_action_data::UserAction;

use crate::actions;
use crate::actions::InterfaceAction;

/// Helper to construct a [StandardAction], a client-opaque serialized action.
#[derive(Default, Clone)]
pub struct ActionBuilder {
    action: Option<UserAction>,
    update: Vec<Command>,
    request_fields: Vec<String>,
}

impl ActionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> StandardAction {
        StandardAction {
            payload: self.action.map_or(vec![], actions::payload),
            update: Some(actions::command_list(None, self.update.clone())),
            request_fields: self
                .request_fields
                .iter()
                .map(|f| (f.clone(), String::new()))
                .collect(),
        }
    }

    /// Sets a server action to perform
    pub fn action(mut self, action: impl Into<UserAction>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Adds a client update to perform immediately
    pub fn update(mut self, command: impl Into<Command>) -> Self {
        self.update.push(command.into());
        self
    }

    /// Adds a named input field whose value should be returned to the server
    /// when this action is being submitted.
    pub fn request_field(mut self, field: impl Into<String>) -> Self {
        self.request_fields.push(field.into());
        self
    }
}

impl InterfaceAction for ActionBuilder {
    fn as_client_action(&self) -> Action {
        Action::StandardAction(self.clone().build())
    }
}
