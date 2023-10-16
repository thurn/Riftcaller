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
pub fn visitation() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(1)).build();
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.hand().len(), 1);
    assert!(g.user.cards.discard_pile().contains_card(CardName::Visitation));
}

#[test]
pub fn visitation_pass() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(1)).build();
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    g.click(Button::NoPromptAction);
    assert_eq!(g.user.cards.hand().len(), 0);
    assert!(g.user.cards.evocations_and_allies().contains_card(CardName::Visitation));
}

#[test]
pub fn visitation_multiple_copies() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(1)).build();
    g.create_and_play(CardName::Visitation);
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    g.click(Button::NoPromptAction);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.hand().len(), 1);
    assert!(g.user.cards.discard_pile().contains_card(CardName::Visitation));
    assert!(g.user.cards.evocations_and_allies().contains_card(CardName::Visitation));
}

#[test]
pub fn visitation_prevent_partial() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(5)).build();
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestSpellDeal5Damage);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.hand().len(), 2);
}

#[test]
pub fn visitation_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(5)).build();
    g.create_and_play_upgraded(CardName::Visitation);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestSpellDeal5Damage);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.hand().len(), 5);
}

#[test]
pub fn empyreal_chorus() {
    let (cost, gained) = (1, 8);
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .room_occupant(test_constants::ROOM_ID, CardName::TestScheme3_10),
        )
        .build();
    let id = g.create_and_play(CardName::EmpyrealChorus);
    g.activate_ability_with_target(id, 0, test_constants::ROOM_ID);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost + gained);
    assert!(!g.user.data.raid_active());
    assert!(!g.user.cards.room_occupants(test_constants::ROOM_ID)[0].revealed_to_me());
}
