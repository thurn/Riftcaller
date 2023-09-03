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

use core_ui::action_builder::ActionBuilder;
use core_ui::actions;
use core_ui::button::Button;
use core_ui::design::FontSize;
use core_ui::list_cell::ListCell;
use core_ui::panel_window::PanelWindow;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use core_ui::text::Text;
use core_ui::text_field::TextField;
use game_data::card_name::{CardName, CardVariant};
use panel_address::{Panel, PanelAddress, StandardPanel};
use user_action_data::{DebugAction, UserAction};

#[derive(Debug)]
pub struct AddToHandPanel {
    filter_string: String,
}

impl AddToHandPanel {
    pub fn new(filter_string: impl Into<String>) -> Self {
        Self { filter_string: filter_string.into() }
    }

    fn matches(&self, name: CardName) -> bool {
        if name.displayed_name().starts_with("Test") {
            return false;
        }

        name.displayed_name()
            .split(' ')
            .any(|part| part.to_lowercase().starts_with(&self.filter_string.to_lowercase()))
    }
}

impl Panel for AddToHandPanel {
    fn address(&self) -> PanelAddress {
        PanelAddress::StandardPanel(StandardPanel::AddToHand)
    }
}

impl Component for AddToHandPanel {
    fn build(self) -> Option<Node> {
        let mut names =
            enum_iterator::all::<CardName>().filter(|n| self.matches(*n)).collect::<Vec<_>>();
        names.sort();
        let mut content = ScrollView::new("Card List")
            .style(Style::new().margin(Edge::Vertical, 16.px()).flex_grow(1.0))
            .child(
                Row::new("Controls")
                    .style(
                        Style::new()
                            .padding(Edge::Horizontal, 32.px())
                            .padding(Edge::Vertical, 16.px()),
                    )
                    .child(Text::new("Filter:").font_size(FontSize::Body))
                    .child(TextField::new("CardVariant").on_field_changed(
                        actions::with_request_fields(
                            UserAction::Debug(DebugAction::FilterCardList),
                            vec!["CardVariant".to_string()],
                        ),
                    )),
            );

        if self.filter_string.len() > 1 {
            content = content.children(names.into_iter().map(|n| {
                ListCell::new(n.displayed_name()).button(
                    Button::new("Add").action(
                        ActionBuilder::new()
                            .action(UserAction::Debug(DebugAction::AddToHand(
                                CardVariant::standard(n),
                            )))
                            .update(self.close()),
                    ),
                )
            }));
        }

        PanelWindow::new(self.address(), 1200.px(), 900.px())
            .title("Add to Hand")
            .show_close_button(true)
            .content(content)
            .build()
    }
}
