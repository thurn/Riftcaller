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
use core_ui::action_builder::ActionBuilder;
use core_ui::actions::InterfaceAction;
use core_ui::button::{IconButton, IconButtonType};
use core_ui::design::{BackgroundColor, FontSize, COIN_COUNT_BORDER};
use core_ui::icons;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::style::Corner;
use core_ui::text::Text;
use game_data::game_actions::{DisplayPreference, GameAction};
use game_data::game_state::GameState;
use game_data::primitives::DeckId;
use game_data::tutorial_data::TutorialMessageKey;
use panel_address::{DeckEditorData, PlayerPanel, StandardPanel};
use player_data::{PlayerActivityKind, PlayerState, PlayerStatus};
use protos::spelldawn::client_debug_command::DebugCommand;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{ClientDebugCommand, FlexAlign, FlexJustify, FlexPosition};

pub struct ScreenOverlay<'a, 'b> {
    player: &'a PlayerState,
    game: Option<&'b GameState>,
    show_close_button: Option<Panels>,
    show_deck_button: bool,
    show_coin_count: bool,
    show_menu_button: bool,
    set_display_preference_button: Option<DisplayPreference>,
}

impl<'a, 'b> ScreenOverlay<'a, 'b> {
    pub fn new(player: &'a PlayerState) -> Self {
        Self {
            player,
            game: None,
            show_close_button: None,
            show_deck_button: player.current_activity().kind() == PlayerActivityKind::Adventure,
            show_coin_count: player.current_activity().kind() == PlayerActivityKind::Adventure,
            show_menu_button: player.current_activity().kind() != PlayerActivityKind::None,
            set_display_preference_button: None,
        }
    }

    pub fn game(mut self, game: Option<&'b GameState>) -> Self {
        self.game = game;
        self
    }

    pub fn show_close_button(mut self, show_close_button: Panels) -> Self {
        self.show_close_button = Some(show_close_button);
        self
    }

    pub fn show_deck_button(mut self, show_deck_button: bool) -> Self {
        self.show_deck_button = show_deck_button;
        self
    }

    pub fn set_display_preference_button(
        mut self,
        set_display_preference_button: Option<DisplayPreference>,
    ) -> Self {
        self.set_display_preference_button = set_display_preference_button;
        self
    }
}

impl<'a, 'b> Component for ScreenOverlay<'a, 'b> {
    fn build(self) -> Option<Node> {
        let activity = self.player.current_activity();
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
                    .child(self.show_close_button.map(|panels| {
                        IconButton::new(icons::CLOSE)
                            .button_type(IconButtonType::DestructiveLarge)
                            .action(panels)
                            .layout(Layout::new().margin(Edge::Left, 16.px()))
                    }))
                    .child(
                        IconButton::new(icons::BUG)
                            .name(&element_names::FEEDBACK_BUTTON)
                            .button_type(IconButtonType::NavBlue)
                            .layout(Layout::new().margin(Edge::All, 12.px()))
                            .action(if cfg!(debug_assertions) {
                                Panels::open(StandardPanel::DebugPanel(
                                    activity.kind(),
                                    activity.side(),
                                ))
                                .as_client_action()
                            } else {
                                ActionBuilder::new()
                                    .update(Command::Debug(ClientDebugCommand {
                                        debug_command: Some(DebugCommand::ShowFeedbackForm(())),
                                    }))
                                    .as_client_action()
                            })
                            .long_press_action(Panels::open(StandardPanel::DebugPanel(
                                activity.kind(),
                                activity.side(),
                            ))),
                    )
                    .child(self.set_display_preference_button.map(set_display_preference_button))
                    .child(self.show_coin_count.then(|| {
                        self.player.adventure.as_ref().map(|adventure| {
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
                        })
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
                                    Panels::open(PlayerPanel::DeckEditor(DeckEditorData::new(
                                        DeckId::Adventure,
                                    )))
                                    .loading(StandardPanel::DeckEditorLoading)
                                } else {
                                    Panels::open(PlayerPanel::DeckEditorPrompt)
                                        .loading(StandardPanel::DeckEditorLoading)
                                },
                            )
                            .layout(Layout::new().margin(Edge::All, 12.px()))
                    }))
                    .child(self.show_menu_button.then(|| {
                        IconButton::new(icons::BARS)
                            .name(&element_names::MENU_BUTTON)
                            .layout(Layout::new().margin(Edge::All, 12.px()))
                            .button_type(IconButtonType::NavBrown)
                            .action(Panels::open(
                                if matches!(self.player.status, Some(PlayerStatus::Playing(_, _))) {
                                    StandardPanel::GameMenu
                                } else {
                                    StandardPanel::AdventureMenu
                                },
                            ))
                    })),
            )
            .build()
    }
}

fn set_display_preference_button(display_preference: DisplayPreference) -> impl Component {
    let icon = match display_preference {
        DisplayPreference::ShowArenaView(true) => icons::EYE_SLASH,
        DisplayPreference::ShowArenaView(false) => icons::EYE,
    };

    IconButton::new(icon)
        .name(&element_names::UNDO_BUTTON)
        .button_type(IconButtonType::NavBlue)
        .layout(Layout::new().margin(Edge::All, 12.px()))
        .action(GameAction::SetDisplayPreference(display_preference))
}
