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
fn equivalent_exchange() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).in_score_area(CardName::TestScheme3_10))
            .opponent(
                TestSide::new(Side::Riftcaller).curses(1).in_score_area(CardName::TestScheme4_20),
            )
            .build();
    assert_eq!(g.me().score(), 10);
    assert_eq!(g.client.other_player.score(), 20);
    g.create_and_play(CardName::EquivalentExchange);
    g.click(Button::SelectForMultipart);
    g.click(Button::SwapCard);
    assert_eq!(g.me().score(), 20);
    assert_eq!(g.client.other_player.score(), 10);
}

#[test]
fn equivalent_exchange_win_game() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant).in_score_area(CardName::TestScheme3_10).bonus_points(40),
    )
    .opponent(TestSide::new(Side::Riftcaller).curses(1).in_score_area(CardName::TestScheme4_20))
    .build();
    assert_eq!(g.me().score(), 50);
    g.create_and_play(CardName::EquivalentExchange);
    g.click(Button::SelectForMultipart);
    g.click(Button::SwapCard);
    assert!(g.is_victory_for_player(Side::Covenant));
}

#[test]
fn equivalent_exchange_no_curse() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).in_score_area(CardName::TestScheme3_10))
            .opponent(TestSide::new(Side::Riftcaller).in_score_area(CardName::TestScheme4_20))
            .build();
    let id = g.add_to_hand(CardName::EquivalentExchange);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
}

#[test]
fn equivalent_exchange_upgraded() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).in_score_area(CardName::TestScheme3_10))
            .opponent(TestSide::new(Side::Riftcaller).in_score_area(CardName::TestScheme4_20))
            .build();
    g.create_and_play_upgraded(CardName::EquivalentExchange);
    g.click(Button::SelectForMultipart);
    g.click(Button::SwapCard);
    assert_eq!(g.me().score(), 20);
}

#[test]
fn equivalent_exchange_no_covenant_scheme() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant))
        .opponent(TestSide::new(Side::Riftcaller).in_score_area(CardName::TestScheme4_20))
        .build();
    let id = g.add_to_hand(CardName::EquivalentExchange);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
}

#[test]
fn equivalent_exchange_no_riftcaller_scheme() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).in_score_area(CardName::TestScheme4_20))
            .build();
    let id = g.add_to_hand(CardName::EquivalentExchange);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
}

#[test]
fn lightbond() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let scheme = g.add_to_hand(CardName::TestScheme3_10);
    g.create_and_play(CardName::Lightbond);
    g.play_card(scheme, g.user_id(), Some(RoomId::RoomA));
    assert!(g
        .client
        .cards
        .room_occupants(RoomId::RoomA)
        .find_card(CardName::TestScheme3_10)
        .is_face_up());
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::RoomA);
    assert_eq!(g.client.cards.opponent_hand().curse_count(), 2);
}

#[test]
fn lightbond_upgraded() {
    let (cost, gained) = (0, 2);
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let scheme = g.add_to_hand(CardName::TestScheme3_10);
    g.create_and_play_upgraded(CardName::Lightbond);
    g.play_card(scheme, g.user_id(), Some(RoomId::RoomA));
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost + gained);
}

#[test]
fn lightbond_recur_from_discard() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let scheme1 = g.add_to_hand(CardName::TestScheme3_10);
    let scheme2 = g.add_to_hand(CardName::TestScheme4_20);
    let lightbond = g.add_to_hand(CardName::Lightbond);
    g.play_card(lightbond, g.user_id(), None);
    g.play_card(scheme1, g.user_id(), Some(RoomId::RoomA));
    g.create_and_play(CardName::TestRitualReturnDiscardToHand);
    g.play_card(lightbond, g.user_id(), None);
    g.play_card(scheme2, g.user_id(), Some(RoomId::RoomB));

    assert!(g
        .client
        .cards
        .room_occupants(RoomId::RoomB)
        .find_card(CardName::TestScheme4_20)
        .is_face_up());
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::RoomA);
    assert_eq!(g.client.cards.opponent_hand().curse_count(), 2);
    g.opponent_click(Button::EndRaid);
    g.initiate_raid(RoomId::RoomB);
    assert_eq!(g.client.cards.opponent_hand().curse_count(), 4);
}

#[test]
fn lightbond_return_scheme_to_hand() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let scheme = g.add_to_hand(CardName::TestScheme3_10);
    let lightbond = g.add_to_hand(CardName::Lightbond);
    g.play_card(lightbond, g.user_id(), None);
    g.play_card(scheme, g.user_id(), Some(RoomId::RoomA));
    g.create_and_play(CardName::TestRitualReturnAllOccupantsToHand);
    let lightbond2 = g.add_to_hand(CardName::Lightbond);
    g.play_card(lightbond2, g.user_id(), None);
    g.play_card(scheme, g.user_id(), Some(RoomId::RoomA));

    assert!(g
        .client
        .cards
        .room_occupants(RoomId::RoomA)
        .find_card(CardName::TestScheme3_10)
        .is_face_up());
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::RoomA);
    assert_eq!(g.client.cards.opponent_hand().curse_count(), 2);
}

#[test]
fn foresee() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .deck_top(CardName::TestInfernalMinion)
            .deck_top(CardName::TestAstralMinion),
    )
    .build();
    g.create_and_play(CardName::Foresee);
    let infernal = g.client.cards.browser().find_card(CardName::TestInfernalMinion).id();
    let astral = g.client.cards.browser().find_card(CardName::TestAstralMinion).id();
    for card_id in g.client.cards.browser().iter().map(|c| c.id()).collect::<Vec<_>>() {
        g.move_card_to_index(card_id, 0);
    }
    g.move_card_to_index(infernal, 4);
    g.move_card_to_index(astral, 4);
    g.click(Button::SubmitCardSelector);
    g.draw_card();
    test_helpers::assert_cards_match(g.client.cards.hand(), vec![CardName::TestAstralMinion]);
    g.draw_card();
    test_helpers::assert_cards_match(
        g.client.cards.hand(),
        vec![CardName::TestAstralMinion, CardName::TestInfernalMinion],
    );
}

#[test]
fn foresee_upgraded() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).deck_top(CardName::TestMortalMinion)).build();
    g.create_and_play_upgraded(CardName::Foresee);
    let mortal = g.client.cards.browser().find_card(CardName::TestMortalMinion).id();
    for card_id in g.client.cards.browser().iter().map(|c| c.id()).collect::<Vec<_>>() {
        g.move_card_to_index(card_id, 2);
    }
    g.move_card_to_index(mortal, 4);
    g.click(Button::SubmitCardSelector);
    test_helpers::assert_cards_match(g.client.cards.hand(), vec![CardName::TestMortalMinion]);
}

#[test]
fn foresee_must_submit_all() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .deck_top(CardName::TestMortalMinion)
            .deck_top(CardName::TestInfernalMinion)
            .deck_top(CardName::TestAstralMinion),
    )
    .build();
    g.create_and_play(CardName::Foresee);
    let id = g.client.cards.browser().find_card(CardName::TestInfernalMinion).id();
    g.move_card_to_index(id, 4);
    assert!(g.click_with_result(Button::SubmitCardSelector).is_err());
}

#[test]
fn dusks_ascension() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let scheme = g.create_and_play_with_target(CardName::TestScheme3_10, RoomId::RoomA);
    g.create_and_play_with_target(CardName::DusksAscension, RoomId::RoomA);
    assert_eq!(g.client.cards.get(scheme).arena_icon(), "3");
    assert!(g.opponent.cards.get(scheme).is_face_up());
    g.pass_turn(Side::Covenant);
    assert_eq!(g.me().score(), 0);
    g.pass_turn(Side::Riftcaller);
    assert_eq!(g.me().score(), 10);
}

#[test]
fn dusks_ascension_progress_again() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let scheme = g.create_and_play_with_target(CardName::TestScheme4_20, RoomId::RoomA);
    g.progress_room(RoomId::RoomA);
    g.create_and_play_with_target(CardName::DusksAscension, RoomId::RoomA);
    assert_eq!(g.client.cards.get(scheme).arena_icon(), "4");
    assert!(g.opponent.cards.get(scheme).is_face_up());
    g.pass_turn(Side::Covenant);
    assert_eq!(g.me().score(), 0);
    g.pass_turn(Side::Riftcaller);
    assert_eq!(g.me().score(), 20);
}
