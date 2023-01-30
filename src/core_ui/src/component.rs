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

use std::fmt::Debug;

use protos::spelldawn::Node;

/// A component is any reusable piece of UI.
///
/// Typically this is a struct that has one or more properties settable via a
/// builder pattern.
///
/// Components can either return another component, typically by invoking its
/// `build` method, or can create and return UI node directly
pub trait Component {
    fn build(self) -> Option<Node>;
}

/// Lets `Option<Component>` be used as a component
impl<T: Component> Component for Option<T> {
    fn build(self) -> Option<Node> {
        if let Some(c) = self {
            c.build()
        } else {
            None
        }
    }
}

/// Helper trait to let components be moved into a `Box`.
pub trait ComponentObject: Component {
    fn build_boxed(self: Box<Self>) -> Option<Node>;
}

impl<T: Component> ComponentObject for T {
    fn build_boxed(self: Box<Self>) -> Option<Node> {
        self.build()
    }
}

/// Empty component which never renders
#[derive(Debug)]
pub struct EmptyComponent;

impl Component for EmptyComponent {
    fn build(self) -> Option<Node> {
        None
    }
}
