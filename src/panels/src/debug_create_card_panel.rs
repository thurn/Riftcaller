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
use core_ui::panel_window::PanelWindow;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use game_data::card_name::CardMetadata;
use game_data::card_state::CardPosition;
use game_data::primitives::{RoomId, RoomLocation, Side};
use panel_address::{Panel, PanelAddress, StandardPanel};
use protos::spelldawn::{FlexAlign, FlexJustify, FlexWrap};

#[derive(Debug)]
pub struct DebugCreateCardPanel {
    user_side: Side,
    metadata: CardMetadata,
}

impl DebugCreateCardPanel {
    pub fn new(user_side: Side, metadata: CardMetadata) -> Self {
        Self { user_side, metadata }
    }

    fn button(&self, label: impl Into<String>, position: CardPosition) -> Button {
        Button::new(label)
            .action(
                Panels::open(StandardPanel::AddToZone(position, self.metadata))
                    .wait_to_load(true)
                    .and_close(self.address()),
            )
            .layout(Layout::new().margin(Edge::All, 8.px()))
    }
}

impl Panel for DebugCreateCardPanel {
    fn address(&self) -> PanelAddress {
        PanelAddress::StandardPanel(StandardPanel::DebugCreateCard(self.user_side, self.metadata))
    }
}

impl Component for DebugCreateCardPanel {
    fn build(self) -> Option<Node> {
        PanelWindow::new(self.address(), 1200.px(), 900.px())
            .title("Create Card")
            .show_close_button(true)
            .content(
                Row::new("CreateButtons")
                    .style(
                        Style::new()
                            .align_items(FlexAlign::Center)
                            .justify_content(FlexJustify::Center)
                            .wrap(FlexWrap::Wrap),
                    )
                    .child(self.button("Deck Top", CardPosition::DeckTop(self.user_side)))
                    .child(self.button("Hand", CardPosition::Hand(self.user_side)))
                    .child(self.button("Discard Pile", CardPosition::DiscardPile(self.user_side)))
                    .child(self.button(
                        "Sanctum",
                        CardPosition::Room(RoomId::Sanctum, RoomLocation::Defender),
                    ))
                    .child(
                        self.button(
                            "Vault",
                            CardPosition::Room(RoomId::Vault, RoomLocation::Defender),
                        ),
                    )
                    .child(self.button(
                        "Crypt",
                        CardPosition::Room(RoomId::Crypts, RoomLocation::Defender),
                    ))
                    .child(self.button(
                        "Outer Defender",
                        CardPosition::Room(RoomId::RoomA, RoomLocation::Defender),
                    ))
                    .child(self.button(
                        "Outer Occupant",
                        CardPosition::Room(RoomId::RoomA, RoomLocation::Occupant),
                    ))
                    .child(self.button("Discard Pile", CardPosition::DiscardPile(self.user_side))),
            )
            .build()
    }
}