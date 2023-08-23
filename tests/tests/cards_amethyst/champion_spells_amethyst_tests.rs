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

use game_data::card_name::CardName;
use game_data::primitives::{RoomId, Side};
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{ObjectPositionBrowser, PlayerName};
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn meditation() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).mana(5)).build();
    assert_eq!(3, g.me().actions());
    g.create_and_play(CardName::Meditation);
    assert_eq!(9, g.me().mana());
    assert_eq!(1, g.me().actions());
    g.create_and_play(CardName::Meditation);
    assert_eq!(13, g.me().mana());
    assert!(g.has(Button::EndTurn));
}

#[test]
fn coup_de_grace() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.play_with_target_room(CardName::CoupDeGrace, RoomId::Vault);
    assert!(g.user.data.raid_active());
    assert_eq!(2, g.user.cards.in_position(Position::Browser(ObjectPositionBrowser {})).count());
    assert_eq!(0, g.user.cards.hand(PlayerName::User).len());
    g.click_on(g.user_id(), "End Raid");
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
}

#[test]
#[should_panic]
fn coup_de_grace_invalid_room() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.play_with_target_room(CardName::CoupDeGrace, test_constants::ROOM_ID);
}

#[test]
fn charged_strike() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.setup_raid_target(test_helpers::minion_for_resonance(test_constants::TEST_RESONANCE));
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    assert_eq!(test_constants::STARTING_MANA - 3, g.me().mana());
    g.play_with_target_room(CardName::ChargedStrike, test_constants::ROOM_ID);
    assert!(g.user.data.raid_active());
    assert_eq!(test_constants::STARTING_MANA - 4, g.me().mana());
    assert_eq!(5, g.user.this_player.bonus_mana());
    assert_eq!(5, g.opponent.other_player.bonus_mana());

    g.opponent_click(Button::Summon);
    g.click_on(g.user_id(), "Test Weapon");
    assert_eq!(test_constants::STARTING_MANA - 4, g.me().mana());
    assert_eq!(4, g.user.this_player.bonus_mana());
    assert_eq!(4, g.opponent.other_player.bonus_mana());
}

#[test]
fn stealth_mission() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.setup_raid_target(test_helpers::minion_for_resonance(test_constants::TEST_RESONANCE));
    assert_eq!(test_constants::STARTING_MANA, g.opponent.this_player.mana());
    g.play_with_target_room(CardName::StealthMission, test_constants::ROOM_ID);
    g.opponent_click(Button::Summon);
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::MINION_COST - 3,
        g.opponent.this_player.mana()
    );
}

#[test]
fn preparation() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).mana(5)).build();
    assert_eq!(3, g.me().actions());
    g.create_and_play(CardName::Preparation);
    assert_eq!(4, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(1, g.me().actions());
    g.create_and_play(CardName::Preparation);
    assert_eq!(8, g.user.cards.hand(PlayerName::User).len());
    assert!(g.has(Button::EndTurn));
}
