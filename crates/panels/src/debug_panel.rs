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

//! The debug panel provides tools for modifying the game state during
//! development. Typically these options should not be available to production
//! users.

use core_ui::actions::InterfaceAction;
use core_ui::button::Button;
use core_ui::panel_window::PanelWindow;
use core_ui::prelude::*;
use core_ui::{icons, panels};
use data::primitives::Side;
use data::user_actions::DebugAction;
use panel_address::{Panel, PanelAddress};
use protos::spelldawn::client_debug_command::DebugCommand;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{ClientDebugCommand, FlexAlign, FlexJustify, FlexWrap};

#[derive(Debug, Default)]
pub struct DebugPanel {}

impl DebugPanel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Panel for DebugPanel {
    fn address(&self) -> PanelAddress {
        PanelAddress::DebugPanel
    }
}

impl Component for DebugPanel {
    fn build(self) -> Option<Node> {
        let close = panels::close(PanelAddress::DebugPanel);

        PanelWindow::new(PanelAddress::DebugPanel, 1024.px(), 600.px())
            .title("Debug Controls")
            .show_close_button(true)
            .content(
                Row::new("DebugButtons")
                    .style(
                        Style::new()
                            .align_items(FlexAlign::Center)
                            .justify_content(FlexJustify::Center)
                            .wrap(FlexWrap::Wrap),
                    )
                    .child(debug_button("New Game (O)", DebugAction::NewGame(Side::Overlord)))
                    .child(debug_button("New Game (C)", DebugAction::NewGame(Side::Champion)))
                    .child(debug_button("Join Game", DebugAction::JoinGame))
                    .child(debug_button(
                        "Show Logs",
                        vec![close, debug_command(DebugCommand::ShowLogs(()))],
                    ))
                    .child(debug_button(format!("+10{}", icons::MANA), DebugAction::AddMana(10)))
                    .child(debug_button(
                        format!("+{}", icons::ACTION),
                        DebugAction::AddActionPoints(1),
                    ))
                    .child(debug_button("+ Point", DebugAction::AddScore(1)))
                    .child(debug_button("Flip View", DebugAction::FlipViewpoint))
                    .child(debug_button(format!("{} 1", icons::SAVE), DebugAction::SaveState(1)))
                    .child(debug_button(format!("{} 1", icons::RESTORE), DebugAction::LoadState(1)))
                    .child(debug_button(format!("{} 2", icons::SAVE), DebugAction::SaveState(1)))
                    .child(debug_button(format!("{} 2", icons::RESTORE), DebugAction::LoadState(1)))
                    .child(debug_button(format!("{} 3", icons::SAVE), DebugAction::SaveState(3)))
                    .child(debug_button(format!("{} 3", icons::RESTORE), DebugAction::LoadState(3)))
                    .child(debug_button(
                        "Overlord AI",
                        panels::open(PanelAddress::SetPlayerName(Side::Overlord)),
                    ))
                    .child(debug_button(
                        "Champion AI",
                        panels::open(PanelAddress::SetPlayerName(Side::Champion)),
                    ))
                    .child(debug_button("Full Collection", DebugAction::FullCollection)),
            )
            .build()
    }
}

fn debug_command(command: DebugCommand) -> Command {
    Command::Debug(ClientDebugCommand { debug_command: Some(command) })
}

fn debug_button(label: impl Into<String>, action: impl InterfaceAction + 'static) -> Button {
    Button::new(label).action(action).layout(Layout::new().margin(Edge::All, 8.px()))
}
