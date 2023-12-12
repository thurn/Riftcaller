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
fn illeas() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::IlleasTheHighSage))
            .build();
    g.create_and_play(CardName::AncestralKnowledge);
    assert_eq!(g.client.cards.hand().len(), 4);
}

#[test]
fn illeas_does_not_trigger_on_action() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::IlleasTheHighSage))
            .build();
    g.draw_card();
    assert_eq!(g.client.cards.hand().len(), 1);
}

#[test]
fn strazihar() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::StraziharTheAllSeeing))
            .current_turn(Side::Covenant)
            .build();
    let id = g.create_and_play(CardName::TestMinionEndRaid);
    g.opponent_click(Button::Reveal);
    assert!(g.client.cards.get(id).revealed_to_me());
}

#[test]
fn strazihar_pay_to_prevent() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::StraziharTheAllSeeing))
            .current_turn(Side::Covenant)
            .build();
    let id = g.create_and_play(CardName::TestMinionEndRaid);
    g.opponent_click(Button::Pay);
    assert!(!g.client.cards.get(id).revealed_to_me());
    assert_eq!(g.client.other_player.mana(), test_constants::STARTING_MANA - 1);
}

#[test]
fn strazihar_insufficient_mana() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::StraziharTheAllSeeing))
            .opponent(TestSide::new(Side::Covenant).mana(0))
            .current_turn(Side::Covenant)
            .build();
    g.create_and_play(CardName::TestMinionEndRaid);
    assert!(g.side_has(Button::Reveal, Side::Covenant));
    assert!(!g.side_has(Button::Pay, Side::Covenant));
}

#[test]
fn strazihar_glimmersong() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::StraziharTheAllSeeing))
            .build();
    let id = g.create_and_play(CardName::Glimmersong);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestMinionEndRaid);
    g.opponent_click(Button::Reveal);
    assert!(g.client.cards.get(id).arena_icon().contains('1'));
}

#[test]
fn merethyl() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::MerethylLoreSeeker))
            .build();
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 3);
}

#[test]
fn merethyl_trigger_twice() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::MerethylLoreSeeker))
            .build();
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);

    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 3);
}

#[test]
fn merethyl_works_with_raid_spell() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::MerethylLoreSeeker))
            .build();
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.create_and_play(CardName::StrikeTheHeart);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 3);
}

#[test]
fn merethyl_effect_does_not_increase_delve_into_darkness() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::MerethylLoreSeeker))
            .build();
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::EndRaid);
    g.create_and_play(CardName::DelveIntoDarkness);
    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 8);
}

#[test]
fn oleus() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::OleusTheWatcher))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_down_defender(RoomId::Sanctum, CardName::TestMinionEndRaid),
        )
        .build();
    g.initiate_raid(RoomId::Sanctum);
    g.opponent_click(Button::Summon);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA + 2);
}

#[test]
fn oleus_trigger_on_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::OleusTheWatcher))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_down_defender(RoomId::Sanctum, CardName::TestMinionEndRaid),
        )
        .current_turn(Side::Covenant)
        .build();
    g.create_and_play(CardName::TestRitualSummonAllMinions);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA + 2);
}

#[test]
fn ellisar() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::EllisarForgekeeper))
            .build();
    let id = g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.activate_ability(id, 0);
    assert_eq!(g.me().actions(), 4);
}

#[test]
fn ellisar_activate_during_raid() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::EllisarForgekeeper))
            .opponent(
                TestSide::new(Side::Covenant)
                    .face_up_defender(RoomId::Vault, CardName::TestMortalMinion),
            )
            .build();
    let id = g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 0);
    assert_eq!(g.me().actions(), 3);
}

#[test]
fn ellisar_resolution_sacrifice() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::EllisarForgekeeper))
            .opponent(
                TestSide::new(Side::Covenant)
                    .face_up_defender(RoomId::Vault, CardName::TestMortalMinion),
            )
            .build();
    g.create_and_play(CardName::Resolution);
    g.initiate_raid(RoomId::Vault);
    g.click_card_name(CardName::Resolution);
    assert_eq!(g.me().actions(), 3);
}
