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

//! Implements the 'chrome' UI elements which display on top of everything else
//! and provide navigation

use constants::ui_constants;
use core_ui::button::{IconButton, IconButtonType};
use core_ui::design::{BackgroundColor, FontSize, COIN_COUNT_BORDER};
use core_ui::icons;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::style::Corner;
use core_ui::text::Text;
use data::player_data::{PlayerData, PlayerStatus};
use data::primitives::DeckId;
use data::tutorial::TutorialMessageKey;
use panel_address::{DeckEditorData, PanelAddress};
use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition};

pub struct ScreenOverlay<'a> {
    player: &'a PlayerData,
    show_close_button: Option<PanelAddress>,
    show_deck_button: bool,
}

impl<'a> ScreenOverlay<'a> {
    pub fn new(player: &'a PlayerData) -> Self {
        Self { player, show_close_button: None, show_deck_button: true }
    }

    pub fn show_close_button(mut self, show_close_button: PanelAddress) -> Self {
        self.show_close_button = Some(show_close_button);
        self
    }

    pub fn show_deck_button(mut self, show_deck_button: bool) -> Self {
        self.show_deck_button = show_deck_button;
        self
    }
}

impl<'a> Component for ScreenOverlay<'a> {
    fn build(self) -> Option<Node> {
        Row::new("Navbar")
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::Left, 1.safe_area_left())
                    .position(Edge::Right, 1.safe_area_right())
                    .position(Edge::Top, 1.safe_area_top())
                    .height(ui_constants::NAVBAR_HEIGHT.px())
                    .align_items(FlexAlign::FlexStart)
                    .justify_content(FlexJustify::SpaceBetween),
            )
            .child(
                Row::new("Left")
                    .style(Style::new().align_items(FlexAlign::Center))
                    .child(self.show_close_button.map(|address| {
                        IconButton::new(icons::CLOSE)
                            .button_type(IconButtonType::DestructiveLarge)
                            .action(Panels::close(address))
                            .layout(Layout::new().margin(Edge::Left, 16.px()))
                    }))
                    .child(
                        IconButton::new(icons::BUG)
                            .name(&element_names::FEEDBACK_BUTTON)
                            .button_type(IconButtonType::NavBlue)
                            .layout(Layout::new().margin(Edge::All, 12.px()))
                            .action(Panels::open(PanelAddress::DebugPanel)),
                    )
                    .child(self.player.adventure.as_ref().map(|adventure| {
                        Row::new("CoinCount")
                            .style(
                                Style::new()
                                    .margin(Edge::Horizontal, 12.px())
                                    .padding(Edge::Horizontal, 8.px())
                                    .height(80.px())
                                    .background_color(BackgroundColor::CoinCountOverlay)
                                    .border_radius(Corner::All, 12.px())
                                    .border_color(Edge::All, COIN_COUNT_BORDER)
                                    .border_width(Edge::All, 1.px()),
                            )
                            .child(
                                Text::new(format!(
                                    "{} <color=yellow>{}</color>",
                                    adventure.coins,
                                    icons::COINS
                                ))
                                .font_size(FontSize::CoinCount),
                            )
                    })),
            )
            .child(
                Row::new("Right")
                    .child(self.show_deck_button.then(|| {
                        IconButton::new(icons::DECK)
                            .name(&element_names::DECK_BUTTON)
                            .button_type(IconButtonType::NavBrown)
                            .action(
                                if self.player.tutorial.has_seen(TutorialMessageKey::DeckEditor) {
                                    Panels::open(PanelAddress::DeckEditor(DeckEditorData::new(
                                        DeckId::Adventure,
                                    )))
                                    .loading(PanelAddress::DeckEditorLoading)
                                } else {
                                    Panels::open(PanelAddress::DeckEditorPrompt)
                                        .loading(PanelAddress::DeckEditorLoading)
                                },
                            )
                            .layout(Layout::new().margin(Edge::All, 12.px()))
                    }))
                    .child(
                        IconButton::new(icons::BARS)
                            .name(&element_names::MENU_BUTTON)
                            .layout(Layout::new().margin(Edge::All, 12.px()))
                            .button_type(IconButtonType::NavBrown)
                            .action(Panels::open(
                                if matches!(self.player.status, Some(PlayerStatus::Playing(_))) {
                                    PanelAddress::GameMenu
                                } else {
                                    PanelAddress::AdventureMenu
                                },
                            )),
                    ),
            )
            .build()
    }
}
