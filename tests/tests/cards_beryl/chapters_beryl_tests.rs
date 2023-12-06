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

// ========================================== //
// ========== Overlord Riftcallers ========== //
// ========================================== //

#[test]
pub fn zain() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).identity(CardName::ZainCunningDiplomat))
            .build();
    g.create_and_play(CardName::TestMinionLoseMana);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - test_constants::MINION_COST + 1);

    g.opponent_click(Button::EndRaid);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::NoWeapon);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - test_constants::MINION_COST + 2);
}

#[test]
pub fn algrak() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).identity(CardName::AlgrakCouncilsEnforcer))
            .build();
    let scheme1 = g.add_to_hand(CardName::TestScheme3_10);
    g.play_card(scheme1, g.user_id(), Some(RoomId::RoomA));
    let scheme2 = g.add_to_hand(CardName::TestScheme4_20);
    g.play_card(scheme2, g.user_id(), Some(RoomId::RoomB));
    g.pass_turn(Side::Overlord);

    g.initiate_raid(RoomId::RoomA);
    assert!(g.side_has(Button::Score, Side::Champion));
    g.opponent_click(Button::Score);
    g.opponent_click(Button::EndRaid);

    g.initiate_raid(RoomId::RoomB);
    assert!(!g.side_has(Button::Score, Side::Champion));
}

#[test]
pub fn eria() {
    let mut g = TestGame::new(
        TestSide::new(Side::Overlord)
            .identity(CardName::EriaTimeConduit)
            .face_up_defender(RoomId::Vault, CardName::TestMinionLoseActionPoints),
    )
    .build();
    let discarded = g.create_and_play(CardName::Test0CostRitual);
    g.pass_turn(Side::Overlord);
    g.initiate_raid(RoomId::Vault);
    g.opponent_click(Button::NoWeapon);
    g.move_selector_card(discarded);
    g.click(Button::SubmitCardSelector);
    g.opponent_click(Button::EndRaid);
    assert!(g.client.cards.deck_top().contains_card(CardName::Test0CostRitual))
}

#[test]
pub fn eria_does_not_trigger_twice() {
    let mut g = TestGame::new(
        TestSide::new(Side::Overlord)
            .identity(CardName::EriaTimeConduit)
            .face_up_defender(RoomId::Vault, CardName::TestMinionLoseActionPoints),
    )
    .build();
    let discarded1 = g.create_and_play(CardName::Test0CostRitual);
    g.create_and_play(CardName::Test0CostRitual);
    g.pass_turn(Side::Overlord);
    g.initiate_raid(RoomId::Vault);
    g.opponent_click(Button::NoWeapon);
    g.move_selector_card(discarded1);
    g.click(Button::SubmitCardSelector);
    g.opponent_click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    g.opponent_click(Button::NoWeapon);
    g.opponent_click(Button::EndRaid);
}

#[test]
pub fn eria_does_not_trigger_with_no_cards_in_crypt() {
    let mut g = TestGame::new(
        TestSide::new(Side::Overlord)
            .identity(CardName::EriaTimeConduit)
            .face_up_defender(RoomId::Vault, CardName::TestMinionLoseActionPoints),
    )
    .build();
    g.pass_turn(Side::Overlord);
    g.initiate_raid(RoomId::Vault);
    g.opponent_click(Button::NoWeapon);
    assert!(!g.has(Button::SubmitCardSelector));
}

#[test]
pub fn vendoc() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).identity(CardName::VendocSeerInStarlight))
            .build();
    g.pass_turn(Side::Overlord);
    g.click(Button::ChooseCardTypeSpell);
    g.create_and_play(CardName::Test0CostSpell);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA + 2);
    g.create_and_play(CardName::Test0CostSpell);
    // Only triggers once
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA + 2);
}

#[test]
pub fn vendoc_play_different_card_first() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).identity(CardName::VendocSeerInStarlight))
            .build();
    g.pass_turn(Side::Overlord);
    g.click(Button::ChooseCardTypeSpell);
    g.create_and_play(CardName::TestEvocation);
    g.create_and_play(CardName::Test0CostSpell);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA + 2);
}
