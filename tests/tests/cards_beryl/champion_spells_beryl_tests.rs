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

#[test]
fn restoration_upgraded() {
    let (cost, reduction) = (1, 2);
    let mut g = TestGame::new(
        TestSide::new(Side::Champion).in_discard_face_up(CardName::TestWeaponAbyssal),
    )
    .build();
    assert!(g.user.cards.left_items().is_empty());
    g.create_and_play_upgraded(CardName::Restoration);
    test_helpers::assert_cards_match(g.user.cards.hand(), vec![CardName::TestWeaponAbyssal]);
    let id = g.user.cards.hand().find_card(CardName::TestWeaponAbyssal);
    g.play_card(id, g.user_id(), None);
    assert!(g.user.cards.hand().is_empty());
    test_helpers::assert_cards_match(g.user.cards.left_items(), vec![CardName::TestWeaponAbyssal]);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA - cost - test_constants::WEAPON_COST + reduction
    );
}

#[test]
fn restoration_upgraded_stacking() {
    let (cost, reduction) = (1, 2);
    let mut g = TestGame::new(
        TestSide::new(Side::Champion)
            .in_discard_face_up(CardName::TestWeaponReduceCostOnSuccessfulRaid),
    )
    .build();
    assert!(g.user.cards.left_items().is_empty());
    g.initiate_raid(RoomId::Crypts);
    g.click(Button::EndRaid);
    g.create_and_play_upgraded(CardName::Restoration);
    let id = g.user.cards.hand().find_card(CardName::TestWeaponReduceCostOnSuccessfulRaid);
    g.play_card(id, g.user_id(), None);
    assert!(g.user.cards.hand().is_empty());
    // Test weapon costs 5 and reduces cost by 2 on raid access
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost - 5 + reduction + 2);
}

#[test]
fn strike_the_heart() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(TestSide::new(Side::Overlord).hand_size(5))
        .build();
    g.create_and_play(CardName::StrikeTheHeart);
    assert_eq!(g.user.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 3);
}

#[test]
fn strike_the_heart_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(TestSide::new(Side::Overlord).hand_size(5))
        .build();
    g.create_and_play_upgraded(CardName::StrikeTheHeart);
    assert_eq!(g.user.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 4);
}

#[test]
fn enduring_radiance() {
    let (cost, return_cost) = (0, 1);
    let mut g = TestGame::new(TestSide::new(Side::Champion).curses(2)).build();
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 2);
    g.create_and_play(CardName::EnduringRadiance);
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 1);
    g.click(Button::ReturnToHand);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost - return_cost);
    let id = g.user.cards.hand().find_card(CardName::EnduringRadiance);
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 0);
}

#[test]
fn enduring_radiance_no_curses() {
    let (cost, return_cost) = (0, 1);
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::EnduringRadiance);
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 0);
    g.click(Button::ReturnToHand);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost - return_cost);
    let id = g.user.cards.hand().find_card(CardName::EnduringRadiance);
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 0);
}

#[test]
fn enduring_radiance_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).curses(2)).build();
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 2);
    g.create_and_play_upgraded(CardName::EnduringRadiance);
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 1);
    let id = g.user.cards.hand().find_card(CardName::EnduringRadiance);
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 0);
}
