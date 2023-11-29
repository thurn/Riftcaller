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

use core_data::game_primitives::{RoomId, Side};
use game_data::card_name::CardName;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn equivalent_exchange() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).in_score_area(CardName::TestScheme3_10))
            .opponent(
                TestSide::new(Side::Champion).curses(1).in_score_area(CardName::TestScheme4_20),
            )
            .build();
    assert_eq!(g.me().score(), 10);
    assert_eq!(g.user.other_player.score(), 20);
    g.create_and_play(CardName::EquivalentExchange);
    g.click(Button::SelectForMultipart);
    g.click(Button::SwapCard);
    assert_eq!(g.me().score(), 20);
    assert_eq!(g.user.other_player.score(), 10);
}

#[test]
fn equivalent_exchange_win_game() {
    let mut g = TestGame::new(
        TestSide::new(Side::Overlord).in_score_area(CardName::TestScheme3_10).bonus_points(40),
    )
    .opponent(TestSide::new(Side::Champion).curses(1).in_score_area(CardName::TestScheme4_20))
    .build();
    assert_eq!(g.me().score(), 50);
    g.create_and_play(CardName::EquivalentExchange);
    g.click(Button::SelectForMultipart);
    g.click(Button::SwapCard);
    assert!(g.is_victory_for_player(Side::Overlord));
}

#[test]
fn equivalent_exchange_no_curse() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).in_score_area(CardName::TestScheme3_10))
            .opponent(TestSide::new(Side::Champion).in_score_area(CardName::TestScheme4_20))
            .build();
    let id = g.add_to_hand(CardName::EquivalentExchange);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
}

#[test]
fn equivalent_exchange_upgraded() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).in_score_area(CardName::TestScheme3_10))
            .opponent(TestSide::new(Side::Champion).in_score_area(CardName::TestScheme4_20))
            .build();
    g.create_and_play_upgraded(CardName::EquivalentExchange);
    g.click(Button::SelectForMultipart);
    g.click(Button::SwapCard);
    assert_eq!(g.me().score(), 20);
}

#[test]
fn equivalent_exchange_no_overlord_scheme() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).in_score_area(CardName::TestScheme4_20))
        .build();
    let id = g.add_to_hand(CardName::EquivalentExchange);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
}

#[test]
fn equivalent_exchange_no_champion_scheme() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).in_score_area(CardName::TestScheme4_20))
            .build();
    let id = g.add_to_hand(CardName::EquivalentExchange);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
}

#[test]
fn lightbond() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let scheme = g.add_to_hand(CardName::TestScheme3_10);
    g.create_and_play(CardName::Lightbond);
    g.play_card(scheme, g.user_id(), Some(RoomId::RoomA));
    assert!(g
        .user
        .cards
        .room_occupants(RoomId::RoomA)
        .find_card(CardName::TestScheme3_10)
        .is_face_up());
    g.pass_turn(Side::Overlord);
    g.initiate_raid(RoomId::RoomA);
    assert_eq!(g.user.cards.opponent_hand().curse_count(), 2);
}

#[test]
fn lightbond_upgraded() {
    let (cost, gained) = (0, 2);
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let scheme = g.add_to_hand(CardName::TestScheme3_10);
    g.create_and_play_upgraded(CardName::Lightbond);
    g.play_card(scheme, g.user_id(), Some(RoomId::RoomA));
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost + gained);
}

#[test]
fn lightbond_recur_from_discard() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let scheme1 = g.add_to_hand(CardName::TestScheme3_10);
    let scheme2 = g.add_to_hand(CardName::TestScheme4_20);
    let lightbond = g.add_to_hand(CardName::Lightbond);
    g.play_card(lightbond, g.user_id(), None);
    g.play_card(scheme1, g.user_id(), Some(RoomId::RoomA));
    g.create_and_play(CardName::TestRitualReturnDiscardToHand);
    g.play_card(lightbond, g.user_id(), None);
    g.play_card(scheme2, g.user_id(), Some(RoomId::RoomB));

    assert!(g
        .user
        .cards
        .room_occupants(RoomId::RoomB)
        .find_card(CardName::TestScheme4_20)
        .is_face_up());
    g.pass_turn(Side::Overlord);
    g.initiate_raid(RoomId::RoomA);
    assert_eq!(g.user.cards.opponent_hand().curse_count(), 2);
    g.opponent_click(Button::EndRaid);
    g.initiate_raid(RoomId::RoomB);
    assert_eq!(g.user.cards.opponent_hand().curse_count(), 4);
}

#[test]
fn lightbond_return_scheme_to_hand() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let scheme = g.add_to_hand(CardName::TestScheme3_10);
    let lightbond = g.add_to_hand(CardName::Lightbond);
    g.play_card(lightbond, g.user_id(), None);
    g.play_card(scheme, g.user_id(), Some(RoomId::RoomA));
    g.create_and_play(CardName::TestRitualReturnAllOccupantsToHand);
    let lightbond2 = g.add_to_hand(CardName::Lightbond);
    g.play_card(lightbond2, g.user_id(), None);
    g.play_card(scheme, g.user_id(), Some(RoomId::RoomA));

    assert!(g
        .user
        .cards
        .room_occupants(RoomId::RoomA)
        .find_card(CardName::TestScheme3_10)
        .is_face_up());
    g.pass_turn(Side::Overlord);
    g.initiate_raid(RoomId::RoomA);
    assert_eq!(g.user.cards.opponent_hand().curse_count(), 2);
}
