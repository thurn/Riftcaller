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

use core_data::game_primitives::Side;
use core_ui::actions::InterfaceAction;
use game_data::game_actions::GameAction;
use insta::assert_snapshot;
use test_utils::summarize::Summary;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;
use user_action_data::{GameOutcome, UserAction};

#[test]
fn resign() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let response = g
        .perform_action(UserAction::GameAction(GameAction::Resign).as_client_action(), g.user_id());
    assert!(!g.client.this_player.can_take_action());
    assert!(!g.client.other_player.can_take_action());
    assert!(g.is_victory_for_player(Side::Riftcaller));
    assert_snapshot!(Summary::run(&response));
}

#[test]
fn leave_game() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.perform(UserAction::GameAction(GameAction::Resign).as_client_action(), g.user_id());
    let response = g
        .perform_action(UserAction::LeaveGame(GameOutcome::Defeat).as_client_action(), g.user_id());
    assert_snapshot!(Summary::run(&response));
}

#[test]
fn draw_all_covenant_cards() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).deck_sizes(3).build();
    g.pass_turn(Side::Covenant);

    loop {
        g.pass_turn(Side::Riftcaller);
        if g.client.cards.hand().real_cards().is_empty() {
            assert!(g.is_victory_for_player(Side::Riftcaller));
            break;
        }

        let card_id = g.client.cards.hand()[0].id();
        g.play_card(card_id, g.user_id(), None);
        g.pass_turn(Side::Covenant);
    }
}

// #[test]
// fn win_game() {
//     let position = TilePosition::new(1, 1);
//     let mut session = TestSessionBuilder::new()
//         .game(TestGame::new(TestSide::new(Side::Covenant).bonus_points(95)))
//         .adventure(TestAdventure::new(Side::Covenant).coins(Coins(500)).
// visiting_position(position))         .build();
//     session.insert_tile_at_position(AdventureEffect::Battle, position);
//
//     session.create_and_play(CardName::TestScheme3_10);
//     session.progress_room(test_constants::ROOM_ID);
//     session.progress_room(test_constants::ROOM_ID);
//     session.pass_turn(Side::Covenant);
//     session.pass_turn(Side::Riftcaller);
//     session.progress_room(test_constants::ROOM_ID);
//     assert!(session.is_victory_for_player(Side::Covenant));
//     assert_eq!(Coins(500), session.current_coins());
//     session.click_on(session.user_id(), "Continue");
//     assert_eq!(Coins(750), session.current_coins());
// }
