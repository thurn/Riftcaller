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
use game_data::card_name::CardName::TestScheme3_10;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn equivalent_exchange() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).in_score_area(TestScheme3_10))
        .opponent(TestSide::new(Side::Champion).curses(1).in_score_area(CardName::TestScheme4_20))
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
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).in_score_area(TestScheme3_10).bonus_points(40))
            .opponent(
                TestSide::new(Side::Champion).curses(1).in_score_area(CardName::TestScheme4_20),
            )
            .build();
    assert_eq!(g.me().score(), 50);
    g.create_and_play(CardName::EquivalentExchange);
    g.click(Button::SelectForMultipart);
    g.click(Button::SwapCard);
    assert!(g.is_victory_for_player(Side::Overlord));
}

#[test]
fn equivalent_exchange_no_curse() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).in_score_area(TestScheme3_10))
        .opponent(TestSide::new(Side::Champion).in_score_area(CardName::TestScheme4_20))
        .build();
    let id = g.add_to_hand(CardName::EquivalentExchange);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
}

#[test]
fn equivalent_exchange_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).in_score_area(TestScheme3_10))
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
