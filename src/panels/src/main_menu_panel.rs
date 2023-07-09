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

//! The main menu is the first panel seen after startup, providing the
//! option to start a new game.

use core_ui::actions::InterfaceAction;
use core_ui::button::{Button, ButtonType};
use core_ui::panel_window::PanelWindow;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use game_data::player_name::{NamedPlayer, PlayerId};
use panel_address::{Panel, PanelAddress, StandardPanel};
use protos::spelldawn::{FlexAlign, FlexJustify};
use user_action_data::{NamedDeck, NewGameAction, NewGameDeck, UserAction};

#[derive(Debug, Default)]
pub struct MainMenuPanel {}

pub const MAIN_MENU_WIDTH: i32 = 800;
pub const MAIN_MENU_HEIGHT: i32 = 600;

impl Panel for MainMenuPanel {
    fn address(&self) -> PanelAddress {
        StandardPanel::MainMenu.into()
    }
}

impl MainMenuPanel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for MainMenuPanel {
    fn build(self) -> Option<Node> {
        PanelWindow::new(StandardPanel::MainMenu, MAIN_MENU_WIDTH.px(), MAIN_MENU_HEIGHT.px())
            .title("Spelldawn")
            .content(
                Column::new("MeuButtons")
                    .style(
                        Style::new()
                            .width(100.pct())
                            .align_items(FlexAlign::Stretch)
                            .justify_content(FlexJustify::Center),
                    )
                    .child(menu_button(
                        "Tutorial",
                        self.close().action(UserAction::NewGame(NewGameAction {
                            deck: NewGameDeck::NamedDeck(NamedDeck::TutorialChampion),
                            opponent: PlayerId::Named(NamedPlayer::TutorialOpponent),
                            tutorial: true,
                            debug_options: None,
                        })),
                    ))
                    .child(menu_button("New Adventure", Panels::open(StandardPanel::SideSelect)))
                    .child(menu_button("Settings", Panels::open(StandardPanel::Settings)))
                    .child(menu_button("About", Panels::open(StandardPanel::About))),
            )
            .build()
    }
}

fn menu_button(label: impl Into<String>, action: impl InterfaceAction + 'static) -> Button {
    Button::new(label)
        .action(action)
        .button_type(ButtonType::Primary)
        .width_mode(WidthMode::Flexible)
        .layout(Layout::new().margin(Edge::All, 16.px()))
}
