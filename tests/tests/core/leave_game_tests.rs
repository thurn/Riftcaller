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

use core_ui::actions::InterfaceAction;
use game_data::game_actions::GameAction;
use game_data::primitives::Side;
use game_data::user_actions::UserAction;
use insta::assert_snapshot;
use test_utils::summarize::Summary;
use test_utils::*;

#[test]
fn resign() {
    let mut g = new_game(Side::Overlord, Args::default());
    let response = g
        .perform_action(UserAction::GameAction(GameAction::Resign).as_client_action(), g.user_id());
    assert!(!g.user.this_player.can_take_action());
    assert!(!g.user.other_player.can_take_action());
    assert!(g.is_victory_for_player(Side::Champion));
    assert_snapshot!(Summary::run(&response));
}

#[test]
fn leave_game() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.perform(UserAction::GameAction(GameAction::Resign).as_client_action(), g.user_id());
    let response = g.perform_action(UserAction::LeaveGame.as_client_action(), g.user_id());
    assert_snapshot!(Summary::run(&response));
}
