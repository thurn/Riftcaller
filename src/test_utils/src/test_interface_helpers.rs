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

use core_ui::icons;
use game_data::player_name::PlayerId;
use game_data::primitives::Side;
use server::server_data::GameResponseOutput;

use crate::client_interface::HasText;
use crate::test_session::TestSession;
use crate::TestSessionHelpers;

pub enum Button {
    Summon,
    NoSummon,
    NoWeapon,
    ProceedToAccess,
    Score,
    EndRaid,
    EndTurn,
    StartTurn,
    DraftPick,
    ShowDeck,
    CloseIcon,
    StartBattle,
}

pub trait TestInterfaceHelpers {
    /// Look for a button in the user interface and invoke its action as the
    /// current user.
    fn click(&mut self, button: Button) -> GameResponseOutput;

    /// Look for a button in the user interface and invoke its action as the
    /// opponent of the current user.
    fn opponent_click(&mut self, button: Button) -> GameResponseOutput;

    /// Clicks on a button in the user interface as the `side` player.
    fn click_as_side(&mut self, button: Button, side: Side) -> GameResponseOutput;

    /// Returns true if the matching button can be found anywhere in the user
    /// interface for the current user.
    fn has(&self, button: Button) -> bool;

    /// Locate a button containing the provided `text` in the provided player's
    /// interface controls and invoke its registered action.
    fn click_on(&mut self, player_id: PlayerId, text: impl Into<String>) -> GameResponseOutput;

    /// Returns true if the provided text can be found anywhere in the user
    /// interface.
    fn has_text(&self, text: impl Into<String>) -> bool;

    /// Returns the number of panels which are currently open
    fn open_panel_count(&self) -> usize;
}

impl TestInterfaceHelpers for TestSession {
    fn click_on(&mut self, player_id: PlayerId, text: impl Into<String>) -> GameResponseOutput {
        let player = self.player(player_id);
        let handlers = player.interface.all_active_nodes().find_handlers(text);
        let action = handlers.expect("Button not found").on_click.expect("OnClick not found");
        self.perform_action(action.action.expect("Action"), player_id).expect("Server Error")
    }

    fn click(&mut self, button: Button) -> GameResponseOutput {
        let text = resolve_button(button);
        self.click_on(self.user_id(), text)
    }

    fn click_as_side(&mut self, button: Button, side: Side) -> GameResponseOutput {
        let id = self.player_id_for_side(side);
        if id == self.user_id() {
            self.click(button)
        } else {
            self.opponent_click(button)
        }
    }

    fn opponent_click(&mut self, button: Button) -> GameResponseOutput {
        let text = resolve_button(button);
        self.click_on(self.opponent_id(), text)
    }

    fn has(&self, button: Button) -> bool {
        let text = resolve_button(button);
        self.user.interface.all_active_nodes().has_text(text)
    }

    fn has_text(&self, text: impl Into<String>) -> bool {
        self.user.interface.all_active_nodes().has_text(text.into())
    }

    fn open_panel_count(&self) -> usize {
        self.user.interface.panel_count()
    }
}

fn resolve_button(button: Button) -> String {
    match button {
        Button::Summon => "Summon",
        Button::NoSummon => "Pass",
        Button::NoWeapon => "Continue",
        Button::ProceedToAccess => "Proceed",
        Button::Score => "Score",
        Button::EndRaid => "End Raid",
        Button::EndTurn => "End Turn",
        Button::StartTurn => "Start Turn",
        Button::DraftPick => "Pick",
        Button::ShowDeck => icons::DECK,
        Button::CloseIcon => icons::CLOSE,
        Button::StartBattle => "Start",
    }
    .to_string()
}
