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

use core_ui::component::Component;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use protos::riftcaller::{Node, ScrollBarVisibility, TouchScrollBehavior};

use crate::deck_editor_panel::EDITOR_COLUMN_WIDTH;

#[derive(Debug)]
pub struct EditorColumnScroll {
    scroll_view: ScrollView,
}

impl EditorColumnScroll {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for EditorColumnScroll {
    fn default() -> Self {
        Self { scroll_view: ScrollView::new("EditorColumnScroll") }
    }
}

impl HasRenderNode for EditorColumnScroll {
    fn render_node(&mut self) -> &mut Node {
        self.scroll_view.render_node()
    }
}

impl HasNodeChildren for EditorColumnScroll {
    fn get_internal_children(&mut self) -> &mut Vec<Node> {
        self.scroll_view.get_internal_children()
    }
}

impl Component for EditorColumnScroll {
    fn build(self) -> Option<Node> {
        self.scroll_view
            .vertical_scrollbar_visibility(ScrollBarVisibility::Hidden)
            .horizontal_scrollbar_visibility(ScrollBarVisibility::Hidden)
            .touch_scroll_behavior(TouchScrollBehavior::Clamped)
            .scroll_deceleration_rate(0.0)
            .style(Style::new().width(EDITOR_COLUMN_WIDTH.vw()).flex_grow(0.0).flex_shrink(0.0))
            .build()
    }
}
