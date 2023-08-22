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

pub enum Buttons {
    Summon,
    NoSummon,
    NoWeapon,
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
    /// Look for a button in the user interface and invoke its action.
    fn click(&mut self, button: Buttons) -> GameResponseOutput;

    /// Locate a button containing the provided `text` in the provided player's
    /// interface controls and invoke its registered action.
    fn click_on(&mut self, player_id: PlayerId, text: impl Into<String>) -> GameResponseOutput;

    /// Locate a button containing the provided `text` in the provided player's
    /// interface controls and invoke its registered action.
    fn click_button(&mut self, player_id: PlayerId, button: Buttons) -> GameResponseOutput;

    /// Returns true if the matching button can be found anywhere in the user
    /// interface.
    fn has_button(&self, button: Buttons) -> bool;

    /// Returns true if the provided text can be found anywhere in the user
    /// interface.
    fn has_text(&self, text: impl Into<String>) -> bool;

    /// Returns the number of panels which are currently open
    fn open_panel_count(&self) -> usize;
}

impl TestInterfaceHelpers for TestSession {
    fn click(&mut self, button: Buttons) -> GameResponseOutput {
        let (text, side) = resolve_button(button);
        if let Some(s) = side {
            self.click_on(self.player_id_for_side(s), text)
        } else {
            self.click_on(self.user.id, text)
        }
    }

    fn click_on(&mut self, player_id: PlayerId, text: impl Into<String>) -> GameResponseOutput {
        let player = self.player(player_id);
        let handlers = player.interface.all_active_nodes().find_handlers(text);
        let action = handlers.expect("Button not found").on_click.expect("OnClick not found");
        self.perform_action(action.action.expect("Action"), player_id).expect("Server Error")
    }

    fn click_button(&mut self, player_id: PlayerId, button: Buttons) -> GameResponseOutput {
        let (text, _) = resolve_button(button);
        self.click_on(player_id, text)
    }

    fn has_button(&self, button: Buttons) -> bool {
        let (text, side) = resolve_button(button);
        if let Some(s) = side {
            let player = self.player_for_side(s);
            player.interface.all_active_nodes().has_text(text)
        } else {
            self.user.interface.all_active_nodes().has_text(text)
        }
    }

    fn has_text(&self, text: impl Into<String>) -> bool {
        self.user.interface.all_active_nodes().has_text(text.into())
    }

    fn open_panel_count(&self) -> usize {
        self.user.interface.panel_count()
    }
}

fn resolve_button(button: Buttons) -> (String, Option<Side>) {
    match button {
        Buttons::Summon => ("Summon".to_string(), Some(Side::Overlord)),
        Buttons::NoSummon => ("Pass".to_string(), Some(Side::Overlord)),
        Buttons::NoWeapon => ("Continue".to_string(), Some(Side::Champion)),
        Buttons::Score => ("Score".to_string(), Some(Side::Champion)),
        Buttons::EndRaid => ("End Raid".to_string(), Some(Side::Champion)),
        Buttons::EndTurn => ("End Turn".to_string(), None),
        Buttons::StartTurn => ("Start Turn".to_string(), None),
        Buttons::DraftPick => ("Pick".to_string(), None),
        Buttons::ShowDeck => (icons::DECK.to_string(), None),
        Buttons::CloseIcon => (icons::CLOSE.to_string(), None),
        Buttons::StartBattle => ("Start".to_string(), None),
    }
}
