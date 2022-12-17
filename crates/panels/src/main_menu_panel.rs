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
use core_ui::panel::Panel;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use core_ui::{actions, panel};
use data::adventure_actions::AdventureAction;
use data::player_name::{NamedPlayer, PlayerId};
use data::primitives::{DeckIndex, Side};
use data::user_actions::{NewGameAction, UserAction};
use panel_address::PanelAddress;
use protos::spelldawn::{FlexAlign, FlexJustify};

#[derive(Debug, Default)]
pub struct MainMenuPanel {}

impl MainMenuPanel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for MainMenuPanel {
    fn build(self) -> Option<Node> {
        Panel::new(PanelAddress::MainMenu, 600.px(), 600.px())
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
                        actions::close_and(
                            PanelAddress::MainMenu,
                            UserAction::NewGame(NewGameAction {
                                opponent: PlayerId::Named(NamedPlayer::TestAlphaBetaHeuristics),
                                deck_index: DeckIndex { value: 1 },
                                debug_options: None,
                            }),
                        ),
                    ))
                    .child(menu_button(
                        "New Adventure",
                        actions::close_and(
                            PanelAddress::MainMenu,
                            UserAction::AdventureAction(AdventureAction::NewAdventure(
                                Side::Champion,
                            )),
                        ),
                    ))
                    .child(menu_button("Settings", panel::open(PanelAddress::Settings)))
                    .child(menu_button("About", panel::open(PanelAddress::About))),
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
