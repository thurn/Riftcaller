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
use core_ui::icons;
use core_ui::panel_window::PanelWindow;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use game_data::primitives::Side;
use panel_address::{Panel, PanelAddress, StandardPanel};
use player_data::PlayerActivityKind;
use protos::spelldawn::client_debug_command::DebugCommand;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{ClientDebugCommand, FlexAlign, FlexJustify, FlexWrap};
use user_action_data::DebugAction;

#[derive(Debug)]
pub struct DebugPanel {
    activity: PlayerActivityKind,
}

impl DebugPanel {
    pub fn new(activity: PlayerActivityKind) -> Self {
        Self { activity }
    }

    fn main_menu_buttons(&self, row: Row) -> Row {
        let close = Panels::close(self.address());
        row.child(debug_button("New Game (O)", DebugAction::NewGame(Side::Overlord)))
            .child(debug_button("New Game (C)", DebugAction::NewGame(Side::Champion)))
            .child(debug_button(
                "Show Logs",
                vec![close.into(), debug_command(DebugCommand::ShowLogs(()))],
            ))
    }

    fn game_mode_buttons(&self, row: Row) -> Row {
        let close = Panels::close(self.address());
        row.child(debug_button("New Game (O)", DebugAction::NewGame(Side::Overlord)))
            .child(debug_button("New Game (C)", DebugAction::NewGame(Side::Champion)))
            .child(debug_button("Join Game (O)", DebugAction::JoinGame(Side::Overlord)))
            .child(debug_button("Join Game (C)", DebugAction::JoinGame(Side::Champion)))
            .child(debug_button(
                "Show Logs",
                vec![close.into(), debug_command(DebugCommand::ShowLogs(()))],
            ))
            .child(debug_button(format!("+10{}", icons::MANA), DebugAction::AddMana(10)))
            .child(debug_button(format!("+{}", icons::ACTION), DebugAction::AddActionPoints(1)))
            .child(debug_button("+ Point", DebugAction::AddScore(1)))
            .child(debug_button("Flip View", DebugAction::FlipViewpoint))
            .child(debug_button(format!("{} 1", icons::SAVE), DebugAction::SaveGameState(1)))
            .child(debug_button(format!("{} 1", icons::RESTORE), DebugAction::LoadGameState(1)))
            .child(debug_button(format!("{} 2", icons::SAVE), DebugAction::SaveGameState(1)))
            .child(debug_button(format!("{} 2", icons::RESTORE), DebugAction::LoadGameState(1)))
            .child(debug_button(format!("{} 3", icons::SAVE), DebugAction::SaveGameState(3)))
            .child(debug_button(format!("{} 3", icons::RESTORE), DebugAction::LoadGameState(3)))
            .child(debug_button(
                "Overlord AI",
                Panels::open(StandardPanel::SetPlayerName(Side::Overlord))
                    .wait_to_load(true)
                    .and_close(self.address()),
            ))
            .child(debug_button(
                "Champion AI",
                Panels::open(StandardPanel::SetPlayerName(Side::Champion))
                    .wait_to_load(true)
                    .and_close(self.address()),
            ))
    }
}

impl Panel for DebugPanel {
    fn address(&self) -> PanelAddress {
        StandardPanel::DebugPanel(self.activity).into()
    }
}

impl Component for DebugPanel {
    fn build(self) -> Option<Node> {
        let row = Row::new("DebugButtons").style(
            Style::new()
                .align_items(FlexAlign::Center)
                .justify_content(FlexJustify::Center)
                .wrap(FlexWrap::Wrap),
        );
        let content = match self.activity {
            PlayerActivityKind::None => self.main_menu_buttons(row),
            PlayerActivityKind::Adventure => row,
            PlayerActivityKind::PlayingGame => self.game_mode_buttons(row),
        };

        PanelWindow::new(self.address(), 1024.px(), 600.px())
            .title("Debug Controls")
            .show_close_button(true)
            .content(content)
            .build()
    }
}

fn debug_command(command: DebugCommand) -> Command {
    Command::Debug(ClientDebugCommand { debug_command: Some(command) })
}

fn debug_button(label: impl Into<String>, action: impl InterfaceAction + 'static) -> Button {
    Button::new(label).action(action).layout(Layout::new().margin(Edge::All, 8.px()))
}
