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

use game_data::card_name::{CardName, CardVariant};
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
    assert!(g.user.cards.artifacts().is_empty());
    g.create_and_play(CardName::Restoration);
    test_helpers::assert_cards_match(g.user.cards.hand(), vec![CardName::TestWeaponAbyssal]);
    let id = g.user.cards.hand().find_card_id(CardName::TestWeaponAbyssal);
    g.play_card(id, g.user_id(), None);
    assert!(g.user.cards.hand().is_empty());
    test_helpers::assert_cards_match(g.user.cards.artifacts(), vec![CardName::TestWeaponAbyssal]);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost - test_constants::WEAPON_COST);
}

#[test]
fn restoration_no_targets() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(3)).build();
    assert_eq!(g.user.cards.hand().len(), 3);
    let id = g.add_to_hand(CardName::Restoration);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
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
    let id = g.user.cards.hand().find_card_id(CardName::TestWeaponAbyssal);
    g.play_card(id, g.user_id(), None);
    assert!(g.has(Button::EndTurn));
}

#[test]
fn restoration_cannot_take_other_action() {
    let mut g = TestGame::new(
        TestSide::new(Side::Champion).in_discard_face_up(CardName::TestWeaponAbyssal),
    )
    .build();
    assert!(g.user.cards.artifacts().is_empty());
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
    assert!(g.user.cards.artifacts().is_empty());
    g.create_and_play_upgraded(CardName::Restoration);
    test_helpers::assert_cards_match(g.user.cards.hand(), vec![CardName::TestWeaponAbyssal]);
    let id = g.user.cards.hand().find_card_id(CardName::TestWeaponAbyssal);
    g.play_card(id, g.user_id(), None);
    assert!(g.user.cards.hand().is_empty());
    test_helpers::assert_cards_match(g.user.cards.artifacts(), vec![CardName::TestWeaponAbyssal]);
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
    assert!(g.user.cards.artifacts().is_empty());
    g.initiate_raid(RoomId::Crypts);
    g.click(Button::EndRaid);
    g.create_and_play_upgraded(CardName::Restoration);
    let id = g.user.cards.hand().find_card_id(CardName::TestWeaponReduceCostOnSuccessfulRaid);
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
    let id = g.user.cards.hand().find_card_id(CardName::EnduringRadiance);
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
    let id = g.user.cards.hand().find_card_id(CardName::EnduringRadiance);
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 0);
}

#[test]
fn enduring_radiance_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).curses(2)).build();
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 2);
    g.create_and_play_upgraded(CardName::EnduringRadiance);
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 1);
    let id = g.user.cards.hand().find_card_id(CardName::EnduringRadiance);
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.user.cards.hand().count_with_name("Curse"), 0);
}

#[test]
fn sift_the_sands() {
    let (cost, reduction) = (1, 3);
    let mut g =
        TestGame::new(TestSide::new(Side::Champion).deck_top(CardName::TestEvocation)).build();
    g.create_and_play(CardName::SiftTheSands);
    assert_eq!(g.user.cards.hand().len(), 4);
    let id = g.user.cards.find_in_hand(CardVariant::standard(CardName::TestEvocation));
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.user.cards.discard_pile().len(), 4);
    test_helpers::assert_cards_match(
        g.user.cards.evocations_and_allies(),
        vec![CardName::TestEvocation],
    );
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA - cost - (test_constants::EVOCATION_COST - reduction)
    )
}

#[test]
fn sift_the_sands_upgraded() {
    let mut g =
        TestGame::new(TestSide::new(Side::Champion).deck_top(CardName::TestEvocation)).build();
    g.create_and_play_upgraded(CardName::SiftTheSands);
    assert_eq!(g.user.cards.hand().len(), 6);
    let id = g.user.cards.find_in_hand(CardVariant::standard(CardName::TestEvocation));
    g.play_card(id, g.user_id(), None);
}

#[test]
fn holy_aura() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::HolyAura);
    assert_eq!(g.user.cards.hand().len(), 3);
}

#[test]
fn holy_aura_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play_upgraded(CardName::HolyAura);
    assert_eq!(g.user.cards.hand().len(), 4);
}

#[test]
fn holy_aura_discard_to_damage() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.add_to_hand(CardName::HolyAura);
    g.pass_turn(Side::Champion);
    assert_eq!(g.user.cards.hand().len(), 1);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    assert_eq!(g.user.cards.hand().len(), 2);
    test_helpers::assert_cards_match(g.user.cards.discard_pile(), vec![CardName::HolyAura]);
}

#[test]
fn holy_aura_discard_to_hand_size() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(5)).build();
    let id = g.add_to_hand(CardName::HolyAura);
    g.move_to_end_step(Side::Champion);
    g.move_selector_card(id);
    g.click(Button::SubmitDiscard);
    assert_eq!(g.user.cards.hand().len(), 7);
    test_helpers::assert_cards_match(g.user.cards.discard_pile(), vec![CardName::HolyAura]);
}

#[test]
fn holy_aura_discard_to_sift_the_sands() {
    let mut g = TestGame::new(
        TestSide::new(Side::Champion)
            .deck_top(CardName::HolyAura)
            .deck_top(CardName::TestEvocation),
    )
    .build();
    g.create_and_play(CardName::SiftTheSands);
    let id = g.user.cards.find_in_hand(CardVariant::standard(CardName::TestEvocation));
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.user.cards.hand().len(), 2);
}

#[test]
fn voidstep() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();
    g.create_and_play_with_target(CardName::Voidstep, RoomId::Vault);
    assert!(g.has(Button::EndRaid));
    assert!(g.user.data.raid_active());
}

#[test]
fn voidstep_two_defenders() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion)
                .face_up_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();
    g.create_and_play_with_target(CardName::Voidstep, RoomId::Vault);
    assert_eq!(g.user.cards.raid_display().len(), 1);
}

#[test]
fn keensight() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord).room_occupant(RoomId::RoomA, CardName::TestScheme3_10),
        )
        .build();
    assert!(!g.user.cards.room_occupants(RoomId::RoomA)[0].revealed_to_me());
    g.create_and_play_with_target(CardName::Keensight, RoomId::RoomA);
    assert!(g.user.cards.room_occupants(RoomId::RoomA)[0].revealed_to_me());
    g.click(Button::InitiateRaid);
    assert!(g.user.data.raid_active());
}

#[test]
fn ethereal_incursion() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_down_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();
    g.create_and_play_with_target(CardName::EtherealIncursion, RoomId::Vault);
    g.click_as_side(Button::Summon, Side::Overlord);
    g.click(Button::NoWeapon);
    assert!(!g.user.data.raid_active());
    assert!(!g.user.cards.room_defenders(RoomId::Vault)[0].is_face_up())
}
