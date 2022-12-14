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

use core_ui::button::Button;
use core_ui::design::FontSize;
use core_ui::prelude::*;
use core_ui::style;
use core_ui::text::Text;
use data::adventure::DraftData;
use deck_card::DeckCard;

use crate::full_screen_image_panel::FullScreenImagePanel;

pub struct DraftPanel<'a> {
    pub data: &'a DraftData,
}

impl<'a> Component for DraftPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImagePanel::new()
            .image(style::sprite("TPR/EnvironmentsHQ/mountain"))
            .content(Row::new("DraftPanel").children(self.data.choices.iter().map(|choice| {
                Column::new("Choice")
                    .style(Style::new().margin(Edge::All, 32.px()))
                    .child(
                        DeckCard::new(choice.card)
                            .layout(Layout::new().margin(Edge::All, 8.px()))
                            .height(50.vh()),
                    )
                    .child(
                        Text::new(format!("{}x", choice.quantity), FontSize::Headline)
                            .layout(Layout::new().margin(Edge::All, 8.px())),
                    )
                    .child(Button::new("Pick").layout(Layout::new().margin(Edge::All, 8.px())))
            })))
            .build()
    }
}
