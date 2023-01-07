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

use element_names::ElementName;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::update_interface_element_command::InterfaceUpdate;
use protos::spelldawn::{
    AnimateToElementPositionAndDestroy, DestroyAnimationEffect, DestroyElementAnimation,
    UpdateInterfaceElementCommand,
};

use crate::style::DimensionExt;

/// Command to remove all children of an element
pub fn destroy(name: ElementName, effect: DestroyAnimationEffect) -> Command {
    Command::UpdateInterfaceElement(UpdateInterfaceElementCommand {
        element_name: name.into(),
        interface_update: Some(InterfaceUpdate::Destroy(DestroyElementAnimation {
            effects: vec![effect.into()],
            duration: Some(300.milliseconds()),
        })),
    })
}

/// Move the 'source' element to the 'destination' element and then destroy it
pub fn animate_to_position_and_destroy(
    source: ElementName,
    destination: ElementName,
    effect: DestroyAnimationEffect,
) -> Command {
    Command::UpdateInterfaceElement(UpdateInterfaceElementCommand {
        element_name: source.into(),
        interface_update: Some(InterfaceUpdate::AnimateToElementPosition(
            AnimateToElementPositionAndDestroy {
                target_element_name: destination.into(),
                animation: Some(DestroyElementAnimation {
                    effects: vec![effect.into()],
                    duration: Some(300.milliseconds()),
                }),
                fallback_target_element_name: String::new(),
                do_not_clone: false,
            },
        )),
    })
}

/// Command to remove all children of an element
pub fn clear(name: ElementName) -> Command {
    Command::UpdateInterfaceElement(UpdateInterfaceElementCommand {
        element_name: name.into(),
        interface_update: Some(InterfaceUpdate::ClearChildren(())),
    })
}
