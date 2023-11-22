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

use core_data::game_primitives::Side;
use game_data::card_name::CardName;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn overwhelming_power() {
    let (cost, gained) = (10, 15);
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::OverwhelmingPower);
    assert_eq!(test_constants::STARTING_MANA - cost + gained, g.me().mana());
}

#[test]
fn forced_march() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let scheme = g.create_and_play(CardName::TestScheme3_10);
    g.pass_turn(Side::Overlord);
    g.pass_turn(Side::Champion);
    g.create_and_play_with_target(CardName::ForcedMarch, test_constants::ROOM_ID);
    assert_eq!("2", g.user.get_card(scheme).arena_icon());
}

#[test]
#[should_panic]
fn forced_march_same_turn_panic() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestScheme3_10);
    g.create_and_play_with_target(CardName::ForcedMarch, test_constants::ROOM_ID);
}
