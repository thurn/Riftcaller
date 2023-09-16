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
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn restoration() {
    let cost = 1;
    let mut g = TestGame::new(
        TestSide::new(Side::Champion).in_discard_face_up(CardName::TestWeaponAbyssal),
    )
    .build();
    assert!(g.user.cards.left_items().is_empty());
    g.create_and_play(CardName::Restoration);
    test_helpers::assert_cards_match(g.user.cards.hand(), vec![CardName::TestWeaponAbyssal]);
    let id = g.user.cards.hand().find_card(CardName::TestWeaponAbyssal);
    g.play_card(id, g.user_id(), None);
    assert!(g.user.cards.hand().is_empty());
    test_helpers::assert_cards_match(g.user.cards.left_items(), vec![CardName::TestWeaponAbyssal]);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost - test_constants::WEAPON_COST);
}

#[test]
fn restoration_no_targets() {
    let cost = 1;
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(3)).build();
    assert_eq!(g.user.cards.hand().len(), 3);
    g.create_and_play(CardName::Restoration);
    assert!(g.user.cards.hand().is_empty());
    g.click(Button::SkipPlayingCard);
    assert_eq!(g.user.cards.hand().len(), 3);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost);
}

#[test]
fn restoration_last_action_point() {
    let mut g = TestGame::new(
        TestSide::new(Side::Champion).in_discard_face_up(CardName::TestWeaponAbyssal),
    )
    .actions(1)
    .build();
    g.create_and_play(CardName::Restoration);
    test_helpers::assert_cards_match(g.user.cards.hand(), vec![CardName::TestWeaponAbyssal]);
    let id = g.user.cards.hand().find_card(CardName::TestWeaponAbyssal);
    g.play_card(id, g.user_id(), None);
    assert!(g.has(Button::EndTurn));
}

#[test]
fn restoration_cannot_take_other_action() {
    let mut g = TestGame::new(
        TestSide::new(Side::Champion).in_discard_face_up(CardName::TestWeaponAbyssal),
    )
    .build();
    assert!(g.user.cards.left_items().is_empty());
    g.create_and_play(CardName::Restoration);
    test_helpers::assert_cards_match(g.user.cards.hand(), vec![CardName::TestWeaponAbyssal]);
    assert!(g.draw_card_with_result().is_err());
}
