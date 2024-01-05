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
use protos::riftcaller::object_position::Position;
use protos::riftcaller::DrawCardAction;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
pub fn incarnation_of_justice() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::IncarnationOfJustice);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    g.opponent_click(Button::EndRaid);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(g.client.cards.opponent_hand().len(), 0);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(g.client.cards.opponent_hand().len(), 0);
}

#[test]
pub fn incarnation_of_justice_ancestral_knowledge() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::IncarnationOfJustice);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    g.opponent_click(Button::EndRaid);
    g.create_and_play(CardName::AncestralKnowledge);
    assert_eq!(g.client.cards.opponent_hand().len(), 0);
}

#[test]
pub fn sentinel_sphinx() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .face_up_defender(RoomId::Sanctum, CardName::TestMinionEndRaid),
    )
    .build();
    g.create_and_play_with_target(CardName::SentinelSphinx, RoomId::Vault);
    g.pass_turn(Side::Covenant);
    g.create_and_play(CardName::BackupPlan);

    g.initiate_raid(RoomId::Sanctum);
    // Normal minion: can activate backup plan
    assert!(g.opponent.cards.hand().find_ability_card(CardName::BackupPlan).can_play());
    g.opponent_click(Button::NoWeapon);

    g.initiate_raid(RoomId::Vault);
    g.click(Button::Summon);
    // Sentinel Sphinx: cannot activate backup plan
    assert!(!g.opponent.cards.hand().find_ability_card(CardName::BackupPlan).can_play());
}

#[test]
fn sentinel_sphinx_voidstep() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant).face_up_defender(RoomId::Vault, CardName::SentinelSphinx),
        )
        .build();
    g.create_and_play_with_target(CardName::Voidstep, RoomId::Vault);
    assert!(g.has(Button::NoWeapon));
}

#[test]
fn sentinel_sphinx_foebane() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant).face_up_defender(RoomId::Vault, CardName::SentinelSphinx),
        )
        .build();
    g.create_and_play_with_target(CardName::Foebane, RoomId::Vault);
    g.click(Button::ChooseOnPlay);
    g.initiate_raid(RoomId::Vault);
    assert!(!g.has(Button::Evade));
    g.click(Button::NoWeapon);
}

#[test]
fn lawhold_cavalier() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .actions(6)
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::LawholdCavalier),
        )
        .build();

    // 3 Permanents: opponent picks
    let evocation_id = g.create_and_play(CardName::TestEvocation);
    let artifact_id = g.create_and_play(CardName::TestAstralWeapon);
    let ally_id = g.create_and_play(CardName::TestAlly);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::NoWeapon);
    g.click_card_button(g.opponent_id(), ally_id, Button::SelectForMultipart);
    g.click_card_button(g.opponent_id(), evocation_id, Button::SelectForMultipart);
    g.click_card_button(g.user_id(), evocation_id, Button::ReturnToDeck);
    g.click(Button::EndRaid);
    assert!(g.client.cards.deck_top().contains_card(CardName::TestEvocation));

    // Cannot play a permanent from hand after a minion encounter
    let artifact = g.add_to_hand(CardName::TestInfernalWeapon);
    assert!(g.play_card_with_result(artifact, g.user_id(), None).is_err());

    // With only 2 permanents, opponent does not have to pick
    g.initiate_raid(RoomId::Vault);
    g.click(Button::NoWeapon);
    g.click_card_button(g.user_id(), artifact_id, Button::ReturnToDeck);
    g.click(Button::EndRaid);
    assert!(g.client.cards.deck_top().contains_card(CardName::TestAstralWeapon));

    // With 1 permanent, no prompt happens
    g.initiate_raid(RoomId::Vault);
    g.click(Button::NoWeapon);
    g.click(Button::EndRaid);

    g.pass_turn(Side::Riftcaller);
    g.pass_turn(Side::Covenant);

    // Can now play permanents again
    assert!(g.play_card_with_result(artifact, g.user_id(), None).is_ok());
}

#[test]
fn angel_of_unity_gain_mana() {
    let cost = 4;
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play_with_target(CardName::TestMinionEndRaid, RoomId::Vault);
    g.create_and_play_with_target(CardName::AngelOfUnity, RoomId::Vault);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::Summon);
    g.opponent_click(Button::NoWeapon);
    assert!(!g.client.data.raid_active());
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost + 2);
}

#[test]
fn angel_of_unity_upgraded() {
    let cost = 4;
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::TestScheme1_10);
    g.create_and_play_upgraded(CardName::AngelOfUnity);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(test_constants::ROOM_ID);
    g.click(Button::Summon);
    g.opponent_click(Button::NoWeapon);
    assert!(!g.client.data.raid_active());
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost + 2);
}

#[test]
fn angel_of_unity_end_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play_with_target(CardName::AngelOfUnity, RoomId::Vault);
    g.pass_turn(Side::Covenant);
    g.create_and_play(CardName::TestAstralWeapon);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::Summon);
    g.click_on(g.opponent_id(), "Weapon");
    g.activate_ability(id, 0);
    assert!(!g.client.data.raid_active());
    assert!(matches!(g.client.cards.get(id).position(), Position::DiscardPile(..)));
}

#[test]
fn angel_of_unity_cannot_activate_when_played() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play_with_target(CardName::AngelOfUnity, RoomId::Vault);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn angel_of_unity_cannot_activate_before_room_approach() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play_with_target(CardName::AngelOfUnity, RoomId::Vault);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::Summon);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn aeon_swimmer_combat_ability() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .actions(4)
        .opponent(
            TestSide::new(Side::Covenant).face_up_defender(RoomId::Vault, CardName::AeonSwimmer),
        )
        .build();
    g.initiate_raid(RoomId::Vault);
    g.click(Button::NoWeapon);
    assert_eq!(g.me().actions(), 1);
    assert!(!g.client.data.raid_active());
}

#[test]
fn aeon_swimmer_sacrifice_cannot_pay() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .actions(2)
        .opponent(
            TestSide::new(Side::Covenant).face_up_defender(RoomId::Vault, CardName::AeonSwimmer),
        )
        .build();
    g.create_and_play(CardName::TestMortalWeapon);
    g.initiate_raid(RoomId::Vault);
    g.click_card_name(CardName::TestMortalWeapon);
    let id = g.client.cards.room_defenders(RoomId::Vault)[0].id();
    g.opponent_activate_ability(id, 0, None).expect("Error activating");
    assert_eq!(g.me().actions(), 0);
    assert!(!g.client.data.raid_active());
    assert!(matches!(g.client.cards.get(id).position(), Position::DiscardPile(..)));
}

#[test]
fn aeon_swimmer_sacrifice_pay_actions() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .actions(4)
        .opponent(
            TestSide::new(Side::Covenant).face_up_defender(RoomId::Vault, CardName::AeonSwimmer),
        )
        .build();
    g.create_and_play(CardName::TestMortalWeapon);
    g.initiate_raid(RoomId::Vault);
    g.click_card_name(CardName::TestMortalWeapon);
    let id = g.client.cards.room_defenders(RoomId::Vault)[0].id();
    g.opponent_activate_ability(id, 0, None).expect("Error activating");
    g.click(Button::Pay);
    assert_eq!(g.me().actions(), 1);
    assert!(g.client.data.raid_active());
    assert!(matches!(g.client.cards.get(id).position(), Position::DiscardPile(..)));
}

#[test]
fn aeon_swimmer_sacrifice_end_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .actions(4)
        .opponent(
            TestSide::new(Side::Covenant).face_up_defender(RoomId::Vault, CardName::AeonSwimmer),
        )
        .build();
    g.create_and_play(CardName::TestMortalWeapon);
    g.initiate_raid(RoomId::Vault);
    g.click_card_name(CardName::TestMortalWeapon);
    let id = g.client.cards.room_defenders(RoomId::Vault)[0].id();
    g.opponent_activate_ability(id, 0, None).expect("Error activating");
    g.click(Button::EndRaid);
    assert_eq!(g.me().actions(), 2);
    assert!(!g.client.data.raid_active());
    assert!(matches!(g.client.cards.get(id).position(), Position::DiscardPile(..)));
}

#[test]
fn aeon_swimmer_sacrifice_cannot_activate_different_room() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .actions(4)
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::AngelOfUnity)
                .face_up_defender(RoomId::Vault, CardName::AeonSwimmer),
        )
        .build();
    g.create_and_play(CardName::TestAstralWeapon);
    g.initiate_raid(RoomId::Sanctum);
    g.click_card_name(CardName::TestAstralWeapon);
    let id = g.client.cards.room_defenders(RoomId::Vault)[0].id();
    assert!(g.opponent_activate_ability(id, 0, None).is_err());
}
