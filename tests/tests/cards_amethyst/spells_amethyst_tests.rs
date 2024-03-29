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

use core_data::game_primitives::{RoomId, Side};
use game_data::card_name::CardName;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn meditation() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).mana(5)).build();
    assert_eq!(4, g.me().actions());
    g.create_and_play(CardName::Meditation);
    assert_eq!(9, g.me().mana());
    assert_eq!(2, g.me().actions());
    g.create_and_play(CardName::Meditation);
    assert_eq!(13, g.me().mana());
    assert_eq!(0, g.me().actions());
}

#[test]
fn coup_de_grace() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play_with_target(CardName::CoupDeGrace, RoomId::Vault);
    assert!(g.client.data.raid_active());
    assert_eq!(2, g.client.cards.browser().len());
    assert_eq!(0, g.client.cards.hand().len());
    g.click_on(g.user_id(), "End Raid");
    assert_eq!(1, g.client.cards.hand().len());
}

#[test]
#[should_panic]
fn coup_de_grace_invalid_room() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play_with_target(CardName::CoupDeGrace, test_constants::ROOM_ID);
}

#[test]
fn charged_strike() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.setup_raid_target(test_helpers::minion_for_resonance(test_constants::TEST_RESONANCE));
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    assert_eq!(test_constants::STARTING_MANA - 3, g.me().mana());
    g.create_and_play_with_target(CardName::ChargedStrike, test_constants::ROOM_ID);
    assert!(g.client.data.raid_active());
    assert_eq!(test_constants::STARTING_MANA - 4, g.me().mana());
    assert_eq!(5, g.client.this_player.bonus_mana());
    assert_eq!(5, g.opponent.other_player.bonus_mana());

    g.opponent_click(Button::Summon);
    g.click_on(g.user_id(), "Test Weapon");
    assert_eq!(test_constants::STARTING_MANA - 4, g.me().mana());
    assert_eq!(4, g.client.this_player.bonus_mana());
    assert_eq!(4, g.opponent.other_player.bonus_mana());
}

#[test]
fn stealth_mission() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.setup_raid_target(test_helpers::minion_for_resonance(test_constants::TEST_RESONANCE));
    assert_eq!(test_constants::STARTING_MANA, g.opponent.this_player.mana());
    g.create_and_play_with_target(CardName::StealthMission, test_constants::ROOM_ID);
    g.opponent_click(Button::Summon);
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::MINION_COST - 3,
        g.opponent.this_player.mana()
    );
}

#[test]
fn preparation() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).mana(5)).build();
    assert_eq!(4, g.me().actions());
    g.create_and_play(CardName::Preparation);
    assert_eq!(4, g.client.cards.hand().len());
    assert_eq!(2, g.me().actions());
    g.create_and_play(CardName::Preparation);
    assert_eq!(8, g.client.cards.hand().len());
    assert_eq!(0, g.me().actions());
}
