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

use element_names::ElementNameSelector;
use protos::spelldawn::conditional_query::Query;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{ConditionalCommand, ConditionalQuery};

use crate::actions;

/// Builder to construct Commands which conditionally execute one of two command
/// lists bsaed on a boolean predicate.
pub struct Conditional {
    query: ConditionalQuery,
    if_true: Vec<Command>,
    if_false: Vec<Command>,
}

impl Conditional {
    pub fn if_exists(element: impl ElementNameSelector) -> Self {
        Self {
            query: ConditionalQuery { query: Some(Query::ElementExists(element.selector())) },
            if_true: Vec::new(),
            if_false: Vec::new(),
        }
    }

    pub fn then(mut self, command: impl Into<Command>) -> Self {
        self.if_true.push(command.into());
        self
    }

    pub fn or_else(mut self, command: impl Into<Command>) -> Self {
        self.if_false.push(command.into());
        self
    }
}

impl From<Conditional> for Command {
    fn from(conditional: Conditional) -> Self {
        Command::Conditional(ConditionalCommand {
            query: Some(conditional.query),
            if_true: Some(actions::command_list(conditional.if_true)),
            if_false: Some(actions::command_list(conditional.if_false)),
        })
    }
}
