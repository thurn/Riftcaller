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

use std::sync::atomic::{AtomicU64, Ordering};

use protos::spelldawn::game_command::Command;
use protos::spelldawn::update_interface_element_command::InterfaceUpdate;
use protos::spelldawn::{AnimateToElementPositionAndDestroy, UpdateInterfaceElementCommand};

use crate::style::DimensionExt;

static NEXT: AtomicU64 = AtomicU64::new(1);

/// Represents a unique name for a UI element
#[derive(Clone, Debug)]
pub struct ElementName(String);

impl From<ElementName> for String {
    fn from(name: ElementName) -> Self {
        name.0
    }
}

impl ElementName {
    /// Creates an element with the provided tag appending a unique identifier
    /// to prevent collisions.
    pub fn new(tag: impl Into<String>) -> Self {
        Self(format!("{}{}", tag.into(), NEXT.fetch_add(1, Ordering::SeqCst)))
    }

    /// Creates an element name with the literal provided string, without
    /// providing protection for duplicate elements.
    pub fn constant(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// Command to remove all children of an element
pub fn destroy(name: &ElementName) -> Command {
    Command::UpdateInterfaceElement(UpdateInterfaceElementCommand {
        element_name: name.0.clone(),
        interface_update: Some(InterfaceUpdate::Destroy(())),
    })
}

/// Move the 'source' element to the 'destination' element and then destroy it
pub fn animate_to_position_and_destroy(source: &ElementName, destination: &ElementName) -> Command {
    Command::UpdateInterfaceElement(UpdateInterfaceElementCommand {
        element_name: source.0.clone(),
        interface_update: Some(InterfaceUpdate::AnimateToElementPosition(
            AnimateToElementPositionAndDestroy {
                target_element_name: destination.0.clone(),
                duration: Some(300.milliseconds()),
                fallback_target_element_name: String::new(),
            },
        )),
    })
}

/// Command to remove all children of an element
pub fn clear(name: &ElementName) -> Command {
    Command::UpdateInterfaceElement(UpdateInterfaceElementCommand {
        element_name: name.0.clone(),
        interface_update: Some(InterfaceUpdate::ClearChildren(())),
    })
}
