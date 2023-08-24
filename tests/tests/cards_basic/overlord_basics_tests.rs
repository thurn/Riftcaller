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
use game_data::primitives::Side;
use protos::spelldawn::PlayerName;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::test_session_helpers::TestSessionHelpers;
use test_utils::*;

#[test]
fn conspire() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::Conspire);
    g.level_up_room_times(3);
    assert_eq!(g.me().score(), 10);
}

#[test]
fn devise() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::Devise);
    g.level_up_room_times(4);
    assert_eq!(g.me().score(), 20);
}

#[test]
fn machinate() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::Machinate);
    g.level_up_room_times(5);
    assert_eq!(g.me().score(), 30);
}

#[test]
fn gathering_dark() {
    let (cost, gained) = (5, 9);
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::GatheringDark);
    assert_eq!(test_constants::STARTING_MANA - cost + gained, g.me().mana());
}

#[test]
fn coinery() {
    let (card_cost, taken) = (2, 3);
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::Coinery);
    g.unveil_card(id);
    g.activate_ability(id, 1);
    assert_eq!(test_constants::STARTING_MANA - card_cost + taken, g.me().mana());
    assert!(g.user.get_card(id).is_face_up());
    g.activate_ability(id, 1);
    assert_eq!(test_constants::STARTING_MANA - card_cost + (taken * 2), g.me().mana());
}

#[test]
fn leyline() {
    let (card_cost, gained) = (2, 1);
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::Leyline);
    g.pass_turn(Side::Overlord);
    g.to_end_step(Side::Champion);
    g.unveil_card(id);
    g.click(Button::StartTurn);
    assert_eq!(test_constants::STARTING_MANA - card_cost + gained, g.me().mana());
    g.pass_turn(Side::Overlord);
    g.pass_turn(Side::Champion);
    assert_eq!(test_constants::STARTING_MANA - card_cost + gained * 2, g.me().mana());
    g.pass_turn(Side::Overlord);
    g.pass_turn(Side::Champion);
    assert_eq!(test_constants::STARTING_MANA - card_cost + gained * 3, g.me().mana());
}

#[test]
fn ore_refinery() {
    let (card_cost, stored, taken) = (4, 12, 3);
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::OreRefinery);
    assert_eq!(test_constants::STARTING_MANA, g.me().mana());
    g.pass_turn(Side::Overlord);
    g.to_end_step(Side::Champion);
    g.unveil_card(id);
    assert_eq!(test_constants::STARTING_MANA - card_cost, g.me().mana());
    g.click(Button::StartTurn);
    assert_eq!(test_constants::STARTING_MANA - card_cost + taken, g.me().mana());
    assert_eq!((stored - taken).to_string(), g.user.get_card(id).arena_icon());
}

#[test]
fn crab() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::Crab);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    assert!(!g.user.data.raid_active());
}

#[test]
fn fire_goblin() {
    let (cost, gained) = (1, 1);
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(1))
        .build();
    g.create_and_play(CardName::FireGoblin);
    g.set_up_minion_combat();
    assert_eq!(test_constants::STARTING_MANA - cost, g.me().mana());
    g.opponent_click(Button::NoWeapon);
    assert_eq!(test_constants::STARTING_MANA - cost + gained, g.me().mana());
    assert_eq!(1, g.opponent.cards.discard_pile(PlayerName::User).len());
}

#[test]
fn toucan() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::Toucan);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    assert!(!g.user.data.raid_active());
}

#[test]
fn frog() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::Frog);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    assert!(!g.user.data.raid_active());
}

#[test]
fn captain() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::Captain);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    assert!(!g.user.data.raid_active());
    assert_eq!(2, g.opponent.this_player.actions());
}

#[test]
fn scout() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::Scout);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    assert!(!g.user.data.raid_active());
    assert_eq!(3, g.opponent.this_player.actions());
}
