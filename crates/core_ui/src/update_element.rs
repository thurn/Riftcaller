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

use protos::spelldawn::game_command::Command;
use protos::spelldawn::update_interface_element_command::InterfaceUpdate;
use protos::spelldawn::UpdateInterfaceElementCommand;

/// Represents a unique name for a UI element
pub struct ElementName(String);

impl From<ElementName> for String {
    fn from(name: ElementName) -> Self {
        name.0
    }
}

impl ElementName {
    pub fn new(string: impl Into<String>) -> Self {
        Self(string.into())
    }
}

/// Command to remove all children of an element
pub fn clear(name: ElementName) -> Command {
    Command::UpdateInterfaceElement(UpdateInterfaceElementCommand {
        element_name: name.0,
        interface_update: Some(InterfaceUpdate::ClearChildren(())),
    })
}
