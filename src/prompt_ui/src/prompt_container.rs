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

use core_ui::prelude::*;
use protos::riftcaller::{FlexAlign, FlexJustify, FlexWrap};

#[derive(Default)]
pub struct PromptContainer {
    children: Vec<Box<dyn ComponentObject>>,
}

impl PromptContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, child: impl Component + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    pub fn children(mut self, children: Vec<Box<dyn ComponentObject>>) -> Self {
        self.children.extend(children);
        self
    }
}

impl Component for PromptContainer {
    fn build(self) -> Option<Node> {
        // Note that if this container takes up too much space it can block mouse events
        // during a game via MouseOverScreenElement(). Be careful to only size it to fit
        // its visible contents.
        Row::new("PromptContainer")
            .style(
                Style::new()
                    .justify_content(FlexJustify::FlexEnd)
                    .align_items(FlexAlign::Center)
                    .flex_grow(0.0)
                    .wrap(FlexWrap::WrapReverse)
                    .margin(Edge::Horizontal, 16.px()),
            )
            .child(Row::new("PromptContainerInner").children_boxed(self.children))
            .build()
    }
}
