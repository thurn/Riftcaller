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

//! The main menu is the first panel seen after startup, providing the
//! option to start a new game.

use core_data::game_primitives::Milliseconds;
use core_ui::actions::InterfaceAction;
use core_ui::design;
use core_ui::design::{Font, FontColor, FontSize};
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::text::Text;
use panel_address::{Panel, PanelAddress, StandardPanel};
use protos::riftcaller::{EasingMode, FlexJustify, FontStyle};

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
        Column::new("MainMenu")
            .style(
                Style::new()
                    .position(Edge::Left, 48.px())
                    .position(Edge::Bottom, 48.px())
                    .justify_content(FlexJustify::FlexEnd),
            )
            .child(menu_button("Play", 0, Panels::open(StandardPanel::Settings)))
            .child(menu_button("Codex", 1, Panels::open(StandardPanel::Settings)))
            .child(menu_button("Get Involved", 2, Panels::open(StandardPanel::Settings)))
            .child(menu_button("Settings", 3, Panels::open(StandardPanel::Settings)))
            .child(menu_button("Quit", 4, Panels::open(StandardPanel::Settings)))
            .build()
    }
}

fn menu_button(
    label: &'static str,
    index: u32,
    action: impl InterfaceAction + 'static,
) -> impl Component {
    Row::new(format!("{}Button", label))
        .style(
            Style::new()
                .position(Edge::Left, (-200).px())
                .height(80.px())
                .margin(Edge::All, 4.px())
                .color(FontColor::MainMenuButton)
                .opacity(0.0)
                .transition_properties(vec!["opacity".to_string()])
                .transition_durations(vec![adapters::time_value(Milliseconds(200))])
                .transition_easing_modes(vec![EasingMode::EaseInCubic])
                .transition_delays(vec![adapters::time_value(Milliseconds(index * 100))]),
        )
        .hover_style(Style::new().color(FontColor::MainMenuButtonHover))
        .pressed_style(Style::new().color(FontColor::MainMenuButtonPress))
        .on_attach_style(Style::new().position(Edge::Left, 4.px()).opacity(1.0))
        .on_click(action)
        .child(
            Text::new(label)
                .font(Font::MainMenuButton)
                .font_size(FontSize::MainMenuButton)
                .font_style(FontStyle::Bold)
                .outline_color(design::BLACK)
                .raw_color(None)
                .outline_width(2.px()),
        )
}
