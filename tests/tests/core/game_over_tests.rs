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

use adventure_data::adventure::Coins;
use core_ui::actions::InterfaceAction;
use game_data::card_name::CardName;
use game_data::game_actions::GameAction;
use game_data::primitives::Side;
use insta::assert_snapshot;
use test_utils::summarize::Summary;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;
use user_action_data::{GameOutcome, UserAction};

#[test]
fn resign() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let response = g
        .perform_action(UserAction::GameAction(GameAction::Resign).as_client_action(), g.user_id());
    assert!(!g.user.this_player.can_take_action());
    assert!(!g.user.other_player.can_take_action());
    assert!(g.is_victory_for_player(Side::Champion));
    assert_snapshot!(Summary::run(&response));
}

#[test]
fn leave_game() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.perform(UserAction::GameAction(GameAction::Resign).as_client_action(), g.user_id());
    let response = g
        .perform_action(UserAction::LeaveGame(GameOutcome::Defeat).as_client_action(), g.user_id());
    assert_snapshot!(Summary::run(&response));
}

#[test]
fn win_game() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).score(95))
        .adventure(AdventureArgs { current_coins: Coins(500), reward: Coins(250) })
        .build();

    g.create_and_play(CardName::TestScheme3_15);
    g.level_up_room(ROOM_ID);
    g.level_up_room(ROOM_ID);
    g.spend_actions_until_turn_over(Side::Champion);
    g.level_up_room(ROOM_ID);
    assert!(g.is_victory_for_player(Side::Overlord));
    assert_eq!(Coins(500), g.current_coins());
    g.click_on_in_panel(g.user_id(), "Continue");
    assert_eq!(Coins(750), g.current_coins());
}
