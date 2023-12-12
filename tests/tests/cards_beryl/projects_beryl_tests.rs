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
use protos::riftcaller::client_action::Action;
use protos::riftcaller::DrawCardAction;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::{Button, TestInterfaceHelpers, TestSessionHelpers};

#[test]
fn magistrates_thronehall() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::MagistratesThronehall);
    g.summon_project(id);
    g.pass_turn(Side::Covenant);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(g.client.cards.opponent_hand().len(), 2);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(g.client.cards.opponent_hand().len(), 2);
}

#[test]
fn magistrates_thronehall_ancestral_knowledge() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::MagistratesThronehall);
    g.summon_project(id);
    g.pass_turn(Side::Covenant);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    g.create_and_play(CardName::AncestralKnowledge);
    assert_eq!(g.client.cards.opponent_hand().len(), 2);
}

#[test]
fn living_stone() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .face_up_room_occupant(RoomId::RoomB, CardName::TestProject2Cost3Raze),
    )
    .opponent(TestSide::new(Side::Riftcaller).hand_size(5))
    .build();
    let id = g.create_and_play_with_target(CardName::LivingStone, RoomId::RoomA);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::RoomB);
    g.summon_project(id);
    assert_eq!(g.client.cards.opponent_hand().len(), 5);
    g.opponent_click(Button::Destroy);
    g.opponent_click(Button::EndRaid);
    assert_eq!(g.client.cards.opponent_hand().len(), 4);
}

#[test]
fn living_stone_triggers_on_self() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant))
        .opponent(TestSide::new(Side::Riftcaller).hand_size(5))
        .build();
    let id = g.create_and_play_with_target(CardName::LivingStone, RoomId::RoomA);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::RoomA);
    g.summon_project(id);
    assert_eq!(g.client.cards.opponent_hand().len(), 5);
    g.opponent_click(Button::Destroy);
    g.opponent_click(Button::EndRaid);
    assert_eq!(g.client.cards.opponent_hand().len(), 4);
}

#[test]
fn living_stone_multiple_copies() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant))
        .opponent(TestSide::new(Side::Riftcaller).hand_size(5))
        .build();
    let id1 = g.create_and_play_with_target(CardName::LivingStone, RoomId::RoomA);
    let id2 = g.create_and_play_with_target(CardName::LivingStone, RoomId::RoomB);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::RoomA);
    g.summon_project(id1);
    g.summon_project(id2);
    assert_eq!(g.client.cards.opponent_hand().len(), 5);
    g.opponent_click(Button::Destroy);
    g.opponent_click(Button::EndRaid);
    assert_eq!(g.client.cards.opponent_hand().len(), 3);
}

#[test]
fn living_stone_triggers_on_sanctum() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).in_hand(CardName::TestProject2Cost3Raze))
            .opponent(TestSide::new(Side::Riftcaller).hand_size(5))
            .build();
    let id = g.create_and_play_with_target(CardName::LivingStone, RoomId::RoomA);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::Sanctum);
    g.summon_project(id);
    assert_eq!(g.client.cards.opponent_hand().len(), 5);
    g.opponent_click(Button::Discard);
    g.opponent_click(Button::EndRaid);
    assert_eq!(g.client.cards.opponent_hand().len(), 4);
}

#[test]
fn living_stone_triggers_on_vault() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).deck_top(CardName::TestProject2Cost3Raze))
            .opponent(TestSide::new(Side::Riftcaller).hand_size(5))
            .build();
    let id = g.create_and_play_with_target(CardName::LivingStone, RoomId::RoomA);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::Vault);
    g.summon_project(id);
    assert_eq!(g.client.cards.opponent_hand().len(), 5);
    g.opponent_click(Button::Discard);
    g.opponent_click(Button::EndRaid);
    assert_eq!(g.client.cards.opponent_hand().len(), 4);
}
