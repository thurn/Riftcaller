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

use core_ui::design::ORANGE_900;
use core_ui::draggable::Draggable;
use core_ui::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct UICard {
    layout: Layout,
}

impl UICard {
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
}

impl Component for UICard {
    fn build(self) -> RenderResult {
        Draggable::new("CardDraggable")
            .child(
                Column::new("UICard").style(
                    self.layout
                        .to_style()
                        .background_color(ORANGE_900)
                        .width(200.px())
                        .height(400.px()),
                ),
            )
            .build()
    }
}
