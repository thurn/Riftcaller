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

use protos::spelldawn::FlexPosition;

use crate::component::EmptyComponent;
use crate::prelude::*;

/// Renders children within the safe area of a mobile device. The area outside
/// of the safe area is left transparent.
pub struct SafeScreen {
    content: Box<dyn ComponentObject>,
}

impl SafeScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(mut self, content: impl Component + 'static) -> Self {
        self.content = Box::new(content);
        self
    }
}

impl Default for SafeScreen {
    fn default() -> Self {
        Self { content: Box::new(EmptyComponent) }
    }
}

impl Component for SafeScreen {
    fn build(self) -> Option<Node> {
        Row::new("SafeScreenBackground")
            .style(Style::new().position_type(FlexPosition::Absolute).position(Edge::All, 0.px()))
            .child(
                Row::new("SafeScreen")
                    .style(
                        Style::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Top, 1.safe_area_top())
                            .position(Edge::Right, 1.safe_area_right())
                            .position(Edge::Bottom, 1.safe_area_bottom())
                            .position(Edge::Left, 1.safe_area_left()),
                    )
                    .child_boxed(self.content),
            )
            .build()
    }
}
