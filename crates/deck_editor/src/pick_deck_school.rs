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

use core_ui::bottom_sheet_content::{BottomSheetButtonType, BottomSheetContent};
use core_ui::button::Button;
use core_ui::design::FontSize;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use core_ui::text::Text;
use data::primitives::Side;
use panel_address::{CreateDeckState, PanelAddress};

pub struct PickDeckSchool {
    side: Side,
}

impl PickDeckSchool {
    pub fn new(side: Side) -> Self {
        Self { side }
    }
}

impl Component for PickDeckSchool {
    fn build(self) -> Option<Node> {
        BottomSheetContent::new()
            .title("School")
            .button_type(BottomSheetButtonType::Back(
                PanelAddress::CreateDeck(CreateDeckState::PickSide).into(),
            ))
            .content(
                Column::new("SchoolChoice")
                    .child(Text::new(format!("Side: {:?}", self.side), FontSize::Headline))
                    .child(Text::new("Pick School:", FontSize::Headline))
                    .child(
                        Row::new("SchoolButtons")
                            .child(
                                Button::new("Law")
                                    .width_mode(WidthMode::Constrained)
                                    .layout(Layout::new().margin(Edge::All, 16.px())),
                            )
                            .child(
                                Button::new("Primal")
                                    .width_mode(WidthMode::Constrained)
                                    .layout(Layout::new().margin(Edge::All, 16.px())),
                            )
                            .child(
                                Button::new("Shadow")
                                    .width_mode(WidthMode::Constrained)
                                    .layout(Layout::new().margin(Edge::All, 16.px())),
                            ),
                    ),
            )
            .build()
    }
}
