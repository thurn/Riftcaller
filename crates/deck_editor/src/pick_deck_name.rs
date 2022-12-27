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
use core_ui::bottom_sheet_content::{BottomSheetButtonType, BottomSheetContent};
use core_ui::button::Button;
use core_ui::design::FontSize;
use core_ui::panel;
use core_ui::prelude::*;
use core_ui::text::Text;
use core_ui::text_field::TextField;
use data::primitives::{School, Side};
use data::user_actions::DeckEditorAction;
use panel_address::{CreateDeckState, PanelAddress, PanelType};

pub const DECK_NAME_INPUT: &str = "DeckNameInput";

pub struct PickDeckName {
    side: Side,
    school: School,
}

impl PickDeckName {
    pub fn new(side: Side, school: School) -> Self {
        Self { side, school }
    }
}

pub fn default_deck_name(side: Side, school: School) -> String {
    format!("{:?} {:?} Deck", side, school)
}

impl PanelType for PickDeckName {}

impl Component for PickDeckName {
    fn build(self) -> Option<Node> {
        BottomSheetContent::new()
            .title("Deck Name")
            .button_type(BottomSheetButtonType::Back(
                PanelAddress::CreateDeck(CreateDeckState::PickSchool(self.side)).into(),
            ))
            .content(
                Column::new("DeckNameChoice")
                    .style(Style::new().width(400.px()))
                    .child(
                        Row::new("NameInput")
                            .style(Style::new())
                            .child(Text::new("Deck Name:").font_size(FontSize::Body))
                            .child(
                                TextField::new(DECK_NAME_INPUT)
                                    .max_characters(1024)
                                    .initial_text(default_deck_name(self.side, self.school)),
                            ),
                    )
                    .child(
                        Text::new(format!("Side: {:?}", self.side)).font_size(FontSize::Headline),
                    )
                    .child(
                        Text::new(format!("School: {:?}", self.school))
                            .font_size(FontSize::Headline),
                    )
                    .child(
                        Button::new("Create Deck").action(
                            ActionBuilder::new()
                                .action(DeckEditorAction::CreateDeck(self.side, self.school))
                                .update(panel::close_bottom_sheet())
                                .request_field(DECK_NAME_INPUT),
                        ),
                    ),
            )
            .build()
    }
}
