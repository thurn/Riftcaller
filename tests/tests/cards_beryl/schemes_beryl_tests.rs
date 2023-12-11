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
fn ethereal_form() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::EtherealForm);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::Score);
    g.opponent_click(Button::EndRaid);
    g.pass_turn(Side::Riftcaller);
    assert_eq!(g.client.other_player.score(), 10);
    assert!(g.client.cards.opponent_score_area().contains_card(CardName::EtherealForm));
    g.activate_ability(id, 0);
    assert_eq!(g.client.other_player.score(), 0);
    assert_eq!(g.client.cards.opponent_score_area().len(), 0);
}

#[test]
fn ethereal_form_cannot_activate_in_play() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::EtherealForm);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn echoing_cacophony() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::EchoingCacophony);
    g.progress_room(test_constants::ROOM_ID);
    g.progress_room(test_constants::ROOM_ID);
    assert_eq!(g.client.cards.opponent_hand().curse_count(), 2);
    g.click(Button::EndTurn);
    assert_eq!(g.client.cards.opponent_hand().curse_count(), 0);
}

#[test]
fn echoing_cacophony_upgraded() {
    let gained = 2;
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play_upgraded(CardName::EchoingCacophony);
    g.progress_room(test_constants::ROOM_ID);
    g.progress_room(test_constants::ROOM_ID);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - 2 + gained);
}

#[test]
fn solidarity() {
    let gained = 7;
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::Solidarity);
    g.progress_room(test_constants::ROOM_ID);
    g.progress_room(test_constants::ROOM_ID);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA + gained - 2);
    assert_eq!(g.client.cards.opponent_display_shelf().leyline_count(), 1);
}

#[test]
fn solidarity_use_mana() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).deck_top(CardName::TestProject2Cost3Raze))
            .build();
    g.create_and_play(CardName::Solidarity);
    g.progress_room(test_constants::ROOM_ID);
    g.progress_room(test_constants::ROOM_ID);
    g.pass_turn(Side::Covenant);
    assert_eq!(g.client.other_player.bonus_mana(), 0);
    g.initiate_raid(RoomId::Vault);
    assert_eq!(g.client.other_player.bonus_mana(), 1);
    g.opponent_click(Button::Discard);
    assert_eq!(g.client.other_player.bonus_mana(), 0);
    assert_eq!(g.client.other_player.mana(), test_constants::STARTING_MANA - (3 - 1));
    g.opponent_click(Button::EndRaid);
    assert_eq!(g.client.other_player.bonus_mana(), 0);
}

#[test]
fn solidarity_two_raids() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).deck_top(CardName::TestProject2Cost3Raze))
            .build();
    g.create_and_play(CardName::Solidarity);
    g.progress_room(test_constants::ROOM_ID);
    g.progress_room(test_constants::ROOM_ID);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::Vault);
    assert_eq!(g.client.other_player.bonus_mana(), 1);
    g.opponent_click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    assert_eq!(g.client.other_player.bonus_mana(), 1);
    g.opponent_click(Button::Discard);
    assert_eq!(g.client.other_player.bonus_mana(), 0);
    g.opponent_click(Button::EndRaid);
}

#[test]
fn solidarity_two_copies() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).deck_top(CardName::TestProject2Cost3Raze))
            .actions(6)
            .build();
    g.create_and_play(CardName::Solidarity);
    g.progress_room(test_constants::ROOM_ID);
    g.progress_room(test_constants::ROOM_ID);
    g.create_and_play(CardName::Solidarity);
    g.progress_room(test_constants::ROOM_ID);
    g.progress_room(test_constants::ROOM_ID);
    g.pass_turn(Side::Covenant);
    assert_eq!(g.client.other_player.bonus_mana(), 0);
    g.initiate_raid(RoomId::Vault);
    assert_eq!(g.client.other_player.bonus_mana(), 2);
    g.opponent_click(Button::Discard);
    assert_eq!(g.client.other_player.bonus_mana(), 0);
    assert_eq!(g.client.other_player.mana(), test_constants::STARTING_MANA - (3 - 2));
    g.opponent_click(Button::EndRaid);
    assert_eq!(g.client.other_player.bonus_mana(), 0);
}

#[test]
fn brilliant_gambit() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::BrilliantGambit);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::Score);
    g.opponent_click(Button::EndRaid);
    assert_eq!(g.client.cards.opponent_display_shelf().leyline_count(), 1);

    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::BrilliantGambit);
    g.progress_room(test_constants::ROOM_ID);
    g.progress_room(test_constants::ROOM_ID);
    assert_eq!(g.client.cards.opponent_display_shelf().leyline_count(), 0);
}
