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
use test_utils::{test_helpers, Button, CardNamesExt, TestInterfaceHelpers, TestSessionHelpers};

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

#[test]
fn sealed_necropolis() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .in_discard_face_up(CardName::TestProject2Cost3Raze)
            .in_discard_face_up(CardName::TestScheme1_10)
            .in_discard_face_up(CardName::TestScheme3_10),
    )
    .build();
    let id = g.create_and_play(CardName::SealedNecropolis);
    g.summon_project(id);
    g.activate_ability(id, 0);
    assert_eq!(g.client.cards.hand().real_cards().len(), 2);

    let scheme1 = g.client.cards.discard_pile().find_card_id(CardName::TestScheme1_10);
    let scheme2 = g.client.cards.discard_pile().find_card_id(CardName::TestScheme3_10);
    g.activate_ability(id, 1);
    assert!(g.client.cards.offscreen().contains_card(CardName::SealedNecropolis));
    g.move_selector_card(scheme1);
    g.move_selector_card(scheme2);
    g.click(Button::SubmitCardSelector);
    test_helpers::assert_cards_match(
        g.client.cards.discard_pile(),
        vec![CardName::TestProject2Cost3Raze],
    );
}

#[test]
fn sealed_necropolis_during_raid() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .in_discard_face_up(CardName::TestProject2Cost3Raze)
            .in_discard_face_up(CardName::TestScheme1_10)
            .in_discard_face_up(CardName::TestScheme3_10),
    )
    .build();
    let id = g.create_and_play(CardName::SealedNecropolis);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(test_constants::ROOM_ID);
    g.summon_project(id);

    // Cannot use action ability
    assert!(g.activate_ability_with_result(id, 0).is_err());

    let scheme1 = g.client.cards.discard_pile().find_card_id(CardName::TestScheme1_10);
    let scheme2 = g.client.cards.discard_pile().find_card_id(CardName::TestScheme3_10);
    g.activate_ability(id, 1);
    assert!(g.client.cards.offscreen().contains_card(CardName::SealedNecropolis));
    g.move_selector_card(scheme1);
    g.move_selector_card(scheme2);
    g.click(Button::SubmitCardSelector);
    test_helpers::assert_cards_match(
        g.client.cards.discard_pile(),
        vec![CardName::TestProject2Cost3Raze],
    );

    // The Raid still active even though the target is gone
    assert!(g.client.data.raid_active());
    g.opponent_click(Button::EndRaid);
}

#[test]
fn sealed_necropolis_during_raid_different_room() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .in_discard_face_up(CardName::TestProject2Cost3Raze)
            .in_discard_face_up(CardName::TestScheme1_10)
            .in_discard_face_up(CardName::TestScheme3_10),
    )
    .build();
    let id = g.create_and_play(CardName::SealedNecropolis);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::Vault);
    g.summon_project(id);

    // Cannot use action ability
    assert!(g.activate_ability_with_result(id, 0).is_err());

    let scheme1 = g.client.cards.discard_pile().find_card_id(CardName::TestScheme1_10);
    g.activate_ability(id, 1);
    assert!(g.client.cards.offscreen().contains_card(CardName::SealedNecropolis));
    g.move_selector_card(scheme1);
    g.click(Button::SubmitCardSelector);
    assert_eq!(g.client.cards.discard_pile().len(), 2);
    g.opponent_click(Button::EndRaid);
}

#[test]
fn haste_resonator_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(4).build();
    let id = g.create_and_play(CardName::HasteResonator);
    g.summon_project(id);
    g.gain_mana();
    g.gain_mana();
    g.gain_mana();
    assert_eq!(g.me().actions(), 1);
}

#[test]
fn haste_resonator_draw() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(4).build();
    let id = g.create_and_play(CardName::HasteResonator);
    g.summon_project(id);
    g.draw_card();
    g.draw_card();
    g.draw_card();
    assert_eq!(g.me().actions(), 1);
}

#[test]
fn haste_resonator_progress() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant).room_occupant(RoomId::RoomB, CardName::TestScheme4_20),
    )
    .actions(4)
    .build();
    let id = g.create_and_play(CardName::HasteResonator);
    g.summon_project(id);
    g.progress_room(RoomId::RoomB);
    g.progress_room(RoomId::RoomB);
    g.progress_room(RoomId::RoomB);
    assert_eq!(g.me().actions(), 1);
}

#[test]
fn haste_resonator_play_cards() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(5).build();
    let id = g.create_and_play(CardName::HasteResonator);
    g.summon_project(id);
    g.create_and_play_with_target(CardName::TestAstralMinion, RoomId::Vault);
    g.create_and_play_with_target(CardName::TestInfernalMinion, RoomId::Sanctum);
    g.create_and_play_with_target(CardName::TestMortalMinion, RoomId::Crypt);
    assert_eq!(g.me().actions(), 2);
}

#[test]
fn haste_resonator_summon_after_actions() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(4).build();
    let id = g.create_and_play(CardName::HasteResonator);
    g.draw_card();
    g.draw_card();
    g.summon_project(id);
    g.draw_card();
    assert_eq!(g.me().actions(), 1);
}

#[test]
fn haste_resonator_play_after_actions() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(4).build();
    g.gain_mana();
    g.gain_mana();
    let id = g.create_and_play(CardName::HasteResonator);
    g.summon_project(id);
    g.gain_mana();
    assert_eq!(g.me().actions(), 1);
}

#[test]
fn haste_resonator_multiple_action_types() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(6).build();
    let id = g.create_and_play(CardName::HasteResonator);
    g.summon_project(id);
    g.draw_card();
    g.draw_card();
    g.draw_card();
    assert_eq!(g.me().actions(), 3);
    g.gain_mana();
    g.gain_mana();
    g.gain_mana();
    assert_eq!(g.me().actions(), 1);
}
