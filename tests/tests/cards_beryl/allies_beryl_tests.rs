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

#[test]
pub fn astrian_oracle() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(TestSide::new(Side::Covenant).hand_size(5))
        .build();
    g.create_and_play(CardName::AstrianOracle);
    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 2);
}

#[test]
pub fn astrian_oracle_two_copies() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(TestSide::new(Side::Covenant).hand_size(5))
        .build();

    g.create_and_play(CardName::AstrianOracle);
    g.create_and_play(CardName::AstrianOracle);

    g.initiate_raid(RoomId::Sanctum);

    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 3);
}

#[test]
pub fn astrian_oracle_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(TestSide::new(Side::Covenant).hand_size(5))
        .build();

    g.create_and_play_upgraded(CardName::AstrianOracle);

    g.initiate_raid(RoomId::Sanctum);

    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 3);
}

#[test]
pub fn resplendent_channeler() {
    let (cost, gained) = (3, 1);
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::ResplendentChanneler);
    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost + gained);
    assert_eq!(g.client.cards.hand().len(), 1);
    g.click(Button::EndRaid);

    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost + gained);
    assert_eq!(g.client.cards.hand().len(), 1);
}

#[test]
pub fn stalwart_protector() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::StalwartProtector);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestSpellGiveCurse);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.hand().curse_count(), 0);
    assert!(g.client.cards.discard_pile().contains_card(CardName::StalwartProtector));
}

#[test]
pub fn stalwart_protector_pass() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::StalwartProtector);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestSpellGiveCurse);
    g.click(Button::NoPromptAction);
    assert_eq!(g.client.cards.hand().curse_count(), 1);
    assert!(g.client.cards.evocations_and_allies().contains_card(CardName::StalwartProtector));
}

#[test]
pub fn stalwart_protector_multiple_copies() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::StalwartProtector);
    g.create_and_play(CardName::StalwartProtector);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestSpellGiveCurse);
    g.click(Button::NoPromptAction);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.hand().curse_count(), 0);
    g.create_and_play(CardName::TestSpellGiveCurse);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.hand().curse_count(), 0);
}

#[test]
pub fn stalwart_protector_activate() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).curses(1)).build();
    let id = g.create_and_play(CardName::StalwartProtector);
    g.activate_ability(id, 1);
    assert_eq!(g.client.cards.hand().curse_count(), 0);
    assert!(g.client.cards.discard_pile().contains_card(CardName::StalwartProtector));
}

#[test]
pub fn stalwart_protector_cannot_activate_with_no_curses() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    let id = g.create_and_play(CardName::StalwartProtector);
    assert!(g.activate_ability_with_result(id, 1).is_err());
}

#[test]
pub fn dawnwarden() {
    let (cost, gained) = (1, 2);
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    let id = g.create_and_play(CardName::Dawnwarden);
    let test_sacrifice = g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.activate_ability(test_sacrifice, 0);
    g.activate_ability(id, 1);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA - test_constants::ARTIFACT_COST - cost + gained
    );
}

#[test]
pub fn spellcraft_ritualist() {
    let cost = 2;
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::SpellcraftRitualist);
    assert_eq!(1, g.client.cards.display_shelf().wound_count());
    g.create_and_play(CardName::TestSpell);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA - cost - (test_constants::SPELL_COST - 1)
    );
    g.create_and_play(CardName::TestSpell);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA - cost - 2 * (test_constants::SPELL_COST - 1)
    );
}
