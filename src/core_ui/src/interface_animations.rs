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

use element_names::{ElementNameSelector, TargetName};
use protos::riftcaller::animate_element_style::Property;
use protos::riftcaller::game_command::Command;
use protos::riftcaller::interface_update::Update;
use protos::riftcaller::{
    AnimateElementStyle, AnimateToPosition, CreateTargetAtChildIndex, EasingMode, ElementAnimation,
    ElementSelector, FlexDisplayStyle, InterfaceUpdate, TimeValue, UpdateInterfaceCommand,
    UpdateInterfaceStep,
};

use crate::prelude::*;

/// Combines a list of [InterfaceAnimation]s into a new single merged animation.
pub fn combine(animations: Vec<InterfaceAnimation>) -> InterfaceAnimation {
    InterfaceAnimation {
        command: UpdateInterfaceCommand {
            steps: animations.into_iter().flat_map(|a| a.command.steps).collect(),
        },
    }
}

/// Animates an element fading to 0 opacity and then removes it
pub fn fade_out(element: impl ElementNameSelector) -> InterfaceAnimation {
    InterfaceAnimation::new()
        .insert(0.milliseconds(), element, AnimateStyle::new(Property::Opacity(0.0)))
        .insert(default_duration(), element, DestroyElement)
}

/// [Command] to toggle whether an interface element is displayed.
pub fn set_displayed(element: impl ElementNameSelector, displayed: bool) -> Command {
    InterfaceAnimation::new().start(element, SetDisplayed { displayed }).into()
}

/// Builder to construct animated updates to user interface elements
#[derive(Default)]
pub struct InterfaceAnimation {
    command: UpdateInterfaceCommand,
}

impl From<InterfaceAnimation> for Command {
    fn from(animation: InterfaceAnimation) -> Self {
        Self::UpdateInterface(animation.command)
    }
}

impl InterfaceAnimation {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an [ElementUpdate] which will run immediately in the animation (at
    /// time zero).
    pub fn start(self, element: impl ElementNameSelector, update: impl ElementUpdate) -> Self {
        self.insert(0.milliseconds(), element, update)
    }

    /// Adds an [ElementUpdate] which will run with the provided `time` delay in
    /// the sequence.
    pub fn insert(
        mut self,
        time: TimeValue,
        element: impl ElementNameSelector,
        update: impl ElementUpdate,
    ) -> Self {
        self.command.steps.push(UpdateInterfaceStep {
            element: Some(element.selector()),
            update: Some(update.build_update()),
            start_time: Some(time),
        });
        self
    }
}

/// Possible mutations to user interface elements  
pub trait ElementUpdate: Sized {
    fn build(self) -> Update;

    fn build_update(self) -> InterfaceUpdate {
        InterfaceUpdate { update: Some(self.build()) }
    }
}

/// Marker trait for interface updates which can be animated
pub trait HasAnimation: Sized + ElementUpdate {
    fn animation(&mut self) -> &mut ElementAnimation;

    fn duration(mut self, duration: TimeValue) -> Self {
        self.animation().duration = Some(duration);
        self
    }

    fn ease(mut self, ease: EasingMode) -> Self {
        self.animation().ease = ease.into();
        self
    }
}

/// Make a copy of this element and set the original to
/// 'visiblity: hidden'. Subsequent selectors in this sequence will
/// apply to the cloned element if they search for an element by name.
#[derive(Default)]
pub struct CloneElement;

impl ElementUpdate for CloneElement {
    fn build(self) -> Update {
        Update::CloneElement(())
    }
}

/// Removes the element from the UI hierarchy entirely
#[derive(Default)]
pub struct DestroyElement;

impl ElementUpdate for DestroyElement {
    fn build(self) -> Update {
        Update::DestroyElement(())
    }
}

pub struct AnimateToElement {
    target: ElementSelector,
    animation: ElementAnimation,
    disable_height_half_offset: bool,
    disable_width_half_offset: bool,
}

impl AnimateToElement {
    pub fn new(target: impl ElementNameSelector) -> Self {
        Self {
            target: target.selector(),
            animation: default_animation(),
            disable_height_half_offset: false,
            disable_width_half_offset: false,
        }
    }

    /// If false, the Y coordinate of the target positon is offset by 1/2
    /// the source element's height.
    pub fn disable_height_half_offset(mut self, disable_height_half_offset: bool) -> Self {
        self.disable_height_half_offset = disable_height_half_offset;
        self
    }

    /// If false, the X coordinate of the target positon is offset by 1/2
    /// the source element's width
    pub fn disable_width_half_offset(mut self, disable_width_half_offset: bool) -> Self {
        self.disable_width_half_offset = disable_width_half_offset;
        self
    }
}

impl HasAnimation for AnimateToElement {
    fn animation(&mut self) -> &mut ElementAnimation {
        &mut self.animation
    }
}

impl ElementUpdate for AnimateToElement {
    fn build(self) -> Update {
        Update::AnimateToPosition(AnimateToPosition {
            destination: Some(self.target),
            animation: Some(self.animation),
            disable_height_half_offset: self.disable_height_half_offset,
            disable_width_half_offset: self.disable_width_half_offset,
        })
    }
}

pub struct SetDisplayed {
    pub displayed: bool,
}

impl ElementUpdate for SetDisplayed {
    fn build(self) -> Update {
        Update::ApplyStyle(
            *Style::new()
                .display(if self.displayed {
                    FlexDisplayStyle::Flex
                } else {
                    FlexDisplayStyle::None
                })
                .wrapped_style(),
        )
    }
}

pub struct AnimateStyle {
    property: Property,
    animation: ElementAnimation,
}

impl AnimateStyle {
    pub fn new(property: Property) -> Self {
        Self { property, animation: default_animation() }
    }
}

impl HasAnimation for AnimateStyle {
    fn animation(&mut self) -> &mut ElementAnimation {
        &mut self.animation
    }
}

impl ElementUpdate for AnimateStyle {
    fn build(self) -> Update {
        Update::AnimateStyle(AnimateElementStyle {
            animation: Some(self.animation),
            property: Some(self.property),
        })
    }
}

pub struct CreateTargetAtIndex {
    parent: ElementSelector,
    name: String,
    index: u32,
    animation: ElementAnimation,
}

impl CreateTargetAtIndex {
    pub fn parent(parent: impl ElementNameSelector) -> Self {
        Self {
            parent: parent.selector(),
            name: "<Target>".to_string(),
            index: 0,
            animation: default_animation(),
        }
    }

    pub fn name(mut self, name: TargetName) -> Self {
        self.name = name.0.into();
        self
    }

    pub fn index(mut self, index: u32) -> Self {
        self.index = index;
        self
    }
}

impl HasAnimation for CreateTargetAtIndex {
    fn animation(&mut self) -> &mut ElementAnimation {
        &mut self.animation
    }
}

impl ElementUpdate for CreateTargetAtIndex {
    fn build(self) -> Update {
        Update::CreateTargetAtChildIndex(CreateTargetAtChildIndex {
            parent: Some(self.parent),
            index: self.index,
            target_name: self.name,
            animation: Some(self.animation),
        })
    }
}

pub fn default_duration() -> TimeValue {
    TimeValue { milliseconds: 300 }
}

fn default_animation() -> ElementAnimation {
    ElementAnimation { duration: Some(default_duration()), ease: EasingMode::Linear.into() }
}
