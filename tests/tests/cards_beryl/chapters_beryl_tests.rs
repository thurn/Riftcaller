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
use test_utils::test_session::TestSession;
use test_utils::*;

#[test]
pub fn nimbus_enclave() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).identity(CardName::NimbusEnclave)).build();
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
pub fn enforcers_of_silence() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).identity(CardName::EnforcersOfSilence)).build();
    let scheme1 = g.add_to_hand(CardName::TestScheme3_10);
    g.play_card(scheme1, g.user_id(), Some(RoomId::RoomA));
    let scheme2 = g.add_to_hand(CardName::TestScheme4_20);
    g.play_card(scheme2, g.user_id(), Some(RoomId::RoomB));
    g.pass_turn(Side::Covenant);

    g.initiate_raid(RoomId::RoomA);
    assert!(g.side_has(Button::Score, Side::Riftcaller));
    g.opponent_click(Button::Score);
    g.opponent_click(Button::EndRaid);

    g.initiate_raid(RoomId::RoomB);
    assert!(!g.side_has(Button::Score, Side::Riftcaller));
}

#[test]
pub fn keepers_of_the_eye() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .identity(CardName::KeepersOfTheEye)
            .face_up_defender(RoomId::Vault, CardName::TestMinionLoseActionPoints),
    )
    .build();
    let discarded = g.create_and_play(CardName::Test0CostRitual);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::Vault);
    g.opponent_click(Button::NoWeapon);
    g.move_selector_card(discarded);
    g.click(Button::SubmitCardSelector);
    g.opponent_click(Button::EndRaid);
    assert!(g.client.cards.deck_top().contains_card(CardName::Test0CostRitual))
}

#[test]
pub fn keepers_of_the_eye_does_not_trigger_twice() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .identity(CardName::KeepersOfTheEye)
            .face_up_defender(RoomId::Vault, CardName::TestMinionLoseActionPoints),
    )
    .build();
    let discarded1 = g.create_and_play(CardName::Test0CostRitual);
    g.create_and_play(CardName::Test0CostRitual);
    g.pass_turn(Side::Covenant);
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
pub fn keepers_of_the_eye_does_not_trigger_with_no_cards_in_crypt() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .identity(CardName::KeepersOfTheEye)
            .face_up_defender(RoomId::Vault, CardName::TestMinionLoseActionPoints),
    )
    .build();
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::Vault);
    g.opponent_click(Button::NoWeapon);
    assert!(!g.has(Button::SubmitCardSelector));
}

#[test]
pub fn the_starseers() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).identity(CardName::TheStarseers)).build();
    g.pass_turn(Side::Covenant);
    g.click(Button::ChooseCardTypeSpell);
    g.create_and_play(CardName::Test0CostSpell);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA + 2);
    g.create_and_play(CardName::Test0CostSpell);
    // Only triggers once
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA + 2);
}

#[test]
pub fn the_starseers_play_different_card_first() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).identity(CardName::TheStarseers)).build();
    g.pass_turn(Side::Covenant);
    g.click(Button::ChooseCardTypeSpell);
    g.create_and_play(CardName::TestEvocation);
    g.create_and_play(CardName::Test0CostSpell);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA + 2);
}

#[test]
pub fn rivers_eye() {
    fn revealed_count(session: &TestSession) -> usize {
        session
            .client
            .cards
            .opponent_hand()
            .real_cards()
            .iter()
            .filter(|c| c.revealed_to_me())
            .count()
    }

    let mut g = TestGame::new(TestSide::new(Side::Covenant).identity(CardName::RiversEye))
        .opponent(TestSide::new(Side::Riftcaller).in_hand(CardName::TestInfernalWeapon))
        .build();
    assert_eq!(revealed_count(&g), 0);
    g.create_and_play(CardName::TestRitualGiveCurse);
    assert_eq!(revealed_count(&g), 1);
    g.pass_turn(Side::Covenant);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(revealed_count(&g), 2);
    g.remove_curse();
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    // Previous cards still revealed
    assert_eq!(revealed_count(&g), 2);
}

#[test]
fn the_conjurers_circle() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).identity(CardName::TheConjurersCircle)).build();
    g.create_and_play_with_target(CardName::TestScheme1_10, RoomId::RoomA);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::RoomA);
    g.opponent_click(Button::Score);
    assert!(g
        .client
        .cards
        .display_shelf()
        .find_card(CardName::TheConjurersCircle)
        .arena_icon()
        .contains('1'));

    g.opponent_click(Button::EndRaid);
    g.pass_turn(Side::Riftcaller);
    let id = g.client.cards.hand().find_ability_card(CardName::TheConjurersCircle).id();
    g.activate_ability(id, 1);
    assert_eq!(g.client.cards.hand().real_cards().len(), 3);
    let play = g.client.cards.hand()[0].id();
    g.play_card(play, g.user_id(), None);
}

#[test]
fn the_conjurers_circle_raze() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .identity(CardName::TheConjurersCircle)
            .in_hand(CardName::TestProject2Cost3Raze),
    )
    .build();
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::Sanctum);
    g.opponent_click(Button::Discard);
    g.opponent_click(Button::EndRaid);
    g.pass_turn(Side::Riftcaller);
    let id = g.client.cards.hand().find_ability_card(CardName::TheConjurersCircle).id();
    g.activate_ability(id, 1);
    assert_eq!(g.client.cards.hand().real_cards().len(), 3);
}

#[test]
fn the_conjurers_circle_does_not_trigger_twice() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant)
            .identity(CardName::TheConjurersCircle)
            .in_hand(CardName::TestProject2Cost3Raze),
    )
    .build();
    g.create_and_play_with_target(CardName::TestScheme1_10, RoomId::RoomA);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(RoomId::Sanctum);
    g.opponent_click(Button::Discard);
    g.opponent_click(Button::EndRaid);
    g.initiate_raid(RoomId::RoomA);
    g.opponent_click(Button::Score);
    g.opponent_click(Button::EndRaid);
    g.pass_turn(Side::Riftcaller);

    assert!(g
        .client
        .cards
        .display_shelf()
        .find_card(CardName::TheConjurersCircle)
        .arena_icon()
        .contains('1'));
}

#[test]
fn the_honorbound() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).identity(CardName::TheHonorbound)).build();
    let scheme = g.create_and_play(CardName::TestScheme3_10);
    g.create_and_play(CardName::TestRitualGiveCurse);
    g.pass_turn(Side::Covenant);
    g.pass_turn(Side::Riftcaller);
    let room_selector = g.client.cards.hand().find_card_id(CardName::TheHonorbound);
    g.play_card(room_selector, g.user_id(), Some(test_constants::ROOM_ID));
    assert!(g.client.cards.get(scheme).arena_icon().contains("1"));
    assert_eq!(g.client.cards.opponent_hand().curse_count(), 0)
}

#[test]
fn the_honorbound_skip() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).identity(CardName::TheHonorbound)).build();
    let scheme = g.create_and_play(CardName::TestScheme3_10);
    g.create_and_play(CardName::TestRitualGiveCurse);
    g.pass_turn(Side::Covenant);
    g.pass_turn(Side::Riftcaller);
    g.click(Button::SkipSelectingRoom);
    assert!(g.client.cards.get(scheme).arena_icon_option().is_none());
    assert_eq!(g.client.cards.opponent_hand().curse_count(), 1)
}

#[test]
fn the_honorbound_score() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).identity(CardName::TheHonorbound)).build();
    g.create_and_play(CardName::TestScheme1_10);
    g.create_and_play(CardName::TestRitualGiveCurse);
    g.pass_turn(Side::Covenant);
    g.pass_turn(Side::Riftcaller);
    let room_selector = g.client.cards.hand().find_card_id(CardName::TheHonorbound);
    g.play_card(room_selector, g.user_id(), Some(test_constants::ROOM_ID));
    assert_eq!(g.client.cards.opponent_hand().curse_count(), 0);
    assert_eq!(g.me().score(), 10);
}
