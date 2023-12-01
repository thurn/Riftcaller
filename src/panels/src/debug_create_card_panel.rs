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

use std::sync::atomic::Ordering;

use core_data::game_primitives::{CardPlayId, RoomId, RoomLocation, Side};
use core_ui::button::Button;
use core_ui::panel_window::PanelWindow;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use game_data::card_name::CardMetadata;
use game_data::card_state::CardPosition;
use game_data::utils;
use panel_address::{Panel, PanelAddress, StandardPanel};
use protos::riftcaller::{FlexAlign, FlexJustify, FlexWrap};

#[derive(Debug)]
pub struct DebugCreateCardPanel {
    user_side: Side,
    metadata: CardMetadata,
}

impl DebugCreateCardPanel {
    pub fn new(user_side: Side, metadata: CardMetadata) -> Self {
        Self { user_side, metadata }
    }

    fn button(
        &self,
        label: impl Into<String>,
        position: CardPosition,
        turn_face_up: bool,
    ) -> Button {
        Button::new(label)
            .action(
                Panels::open(StandardPanel::AddToZone {
                    position,
                    metadata: self.metadata,
                    turn_face_up,
                })
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
        let id = CardPlayId(utils::DEBUG_EVENT_ID.fetch_add(1, Ordering::Relaxed));
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
                    .child(self.button(
                        "User Deck Top",
                        CardPosition::DeckTop(self.user_side),
                        false,
                    ))
                    .child(self.button(
                        "Opponent Deck Top",
                        CardPosition::DeckTop(self.user_side.opponent()),
                        false,
                    ))
                    .child(self.button("User Hand", CardPosition::Hand(self.user_side), false))
                    .child(self.button(
                        "Opponent Hand",
                        CardPosition::Hand(self.user_side.opponent()),
                        false,
                    ))
                    .child(self.button(
                        "User Discard",
                        CardPosition::DiscardPile(self.user_side),
                        false,
                    ))
                    .child(self.button(
                        "Opponent Discard",
                        CardPosition::DiscardPile(self.user_side.opponent()),
                        false,
                    ))
                    .child(self.button(
                        "Sanctum Defender",
                        CardPosition::Room(id, RoomId::Sanctum, RoomLocation::Defender),
                        true,
                    ))
                    .child(self.button(
                        "Vault Defender",
                        CardPosition::Room(id, RoomId::Vault, RoomLocation::Defender),
                        true,
                    ))
                    .child(self.button(
                        "Crypt Defender",
                        CardPosition::Room(id, RoomId::Crypt, RoomLocation::Defender),
                        true,
                    ))
                    .child(self.button(
                        "Outer Defender",
                        CardPosition::Room(id, RoomId::RoomA, RoomLocation::Defender),
                        true,
                    ))
                    .child(self.button(
                        "Outer Face-Down Defender",
                        CardPosition::Room(id, RoomId::RoomA, RoomLocation::Defender),
                        false,
                    ))
                    .child(self.button(
                        "Outer Occupant",
                        CardPosition::Room(id, RoomId::RoomA, RoomLocation::Occupant),
                        false,
                    ))
                    .child(self.button("User Scored", CardPosition::Scored(self.user_side), true))
                    .child(self.button(
                        "Opponent Scored",
                        CardPosition::Scored(self.user_side.opponent()),
                        true,
                    ))
                    .child(self.button(
                        "User Riftcaller",
                        CardPosition::Riftcaller(self.user_side),
                        true,
                    ))
                    .child(self.button(
                        "Opponent Riftcaller",
                        CardPosition::Riftcaller(self.user_side.opponent()),
                        true,
                    )),
            )
            .build()
    }
}
