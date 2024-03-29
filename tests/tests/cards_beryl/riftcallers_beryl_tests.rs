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
fn illea() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::IlleaTheHighSage)).build();
    g.create_and_play(CardName::AncestralKnowledge);
    assert_eq!(g.client.cards.hand().len(), 4);
}

#[test]
fn illea_does_not_trigger_on_action() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::IlleaTheHighSage)).build();
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
fn godmir() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::GodmirSparkOfDefiance))
            .build();
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 3);
}

#[test]
fn godmir_trigger_twice() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::GodmirSparkOfDefiance))
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
fn godmir_works_with_raid_spell() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::GodmirSparkOfDefiance))
            .build();
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.create_and_play(CardName::StrikeTheHeart);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 3);
}

#[test]
fn godmir_effect_does_not_increase_delve_into_darkness() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::GodmirSparkOfDefiance))
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

#[test]
fn seldanna() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::SeldannaRegalPyromancer))
            .opponent(TestSide::new(Side::Covenant).deck_top(CardName::TestProject2Cost3Raze))
            .build();
    g.initiate_raid(RoomId::Vault);
    g.click(Button::Discard);
    assert_eq!(g.client.cards.opponent_discard_pile().len(), 2);
}

#[test]
fn rolant_restoration() {
    let mut g = TestGame::new(
        TestSide::new(Side::Riftcaller)
            .identity(CardName::RolantTheRestorer)
            .in_discard_face_up(CardName::TestAstralWeapon),
    )
    .build();
    g.create_and_play(CardName::Restoration);
    let id = g.client.cards.hand().find_card_id(CardName::TestAstralWeapon);
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.client.cards.hand().len(), 1);
}

#[test]
fn eria() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::EriaTheGhostOfVasilor))
            .opponent(
                TestSide::new(Side::Covenant)
                    .face_up_defender(RoomId::Vault, CardName::TestMinionEndRaid),
            )
            .build();
    g.initiate_raid(RoomId::Vault);
    g.click(Button::NoWeapon);
    assert!(g.client.data.raid_active());
    assert_eq!(g.client.cards.hand().curse_count(), 1);

    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::NoWeapon);
    assert!(!g.client.data.raid_active());
    assert_eq!(g.client.cards.hand().curse_count(), 1);
}

#[test]
fn usilyna_planar_sanctuary() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::UsilynaMasterArtificer))
            .build();
    let id = g.create_and_play(CardName::PlanarSanctuary);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::AddPowerCharges);
    assert!(g.client.cards.get(id).arena_icon().contains("1"));
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    assert!(g.client.cards.get(id).arena_icon().contains("1"));
    g.click(Button::EndRaid);
    g.activate_ability(id, 1);
    assert_eq!(g.client.cards.hand().real_cards().len(), 1);
}

#[test]
fn usilyna_spear_of_conquest() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::UsilynaMasterArtificer))
            .build();
    g.create_and_play(CardName::PlanarSanctuary);
    let spear = g.create_and_play(CardName::SpearOfConquest);
    g.initiate_raid(RoomId::Vault);
    g.click_card_button(g.user_id(), spear, Button::AddPowerCharges);
    assert!(g.client.cards.get(spear).arena_icon().contains("1"));
}

#[test]
fn usilyna_no_targets() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::UsilynaMasterArtificer))
            .build();
    g.create_and_play(CardName::TestAstralWeapon);
    g.initiate_raid(RoomId::Vault);
    assert!(!g.has(Button::AddPowerCharges));
}

#[test]
fn sariandi() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::SariandiPhaseWalker))
            .opponent(
                TestSide::new(Side::Covenant)
                    .face_up_defender(RoomId::Vault, CardName::TestMinionEndRaid)
                    .deck_top(CardName::TestScheme3_10),
            )
            .build();
    let id = g.client.cards.hand().find_ability_card(CardName::SariandiPhaseWalker).id();
    g.activate_ability(id, 0);
    g.click(Button::AccessVault);
    g.click(Button::Score);
    assert_eq!(g.me().score(), 10);
}

#[test]
fn sariandi_once_per_turn() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::SariandiPhaseWalker))
            .opponent(
                TestSide::new(Side::Covenant)
                    .face_up_defender(RoomId::Vault, CardName::TestMinionEndRaid)
                    .deck_top(CardName::TestScheme3_10),
            )
            .build();
    let id = g.client.cards.hand().find_ability_card(CardName::SariandiPhaseWalker).id();
    g.activate_ability(id, 0);
    g.click(Button::AccessVault);
    g.click(Button::EndRaid);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn sariandi_raid_sanctum() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::SariandiPhaseWalker))
            .opponent(
                TestSide::new(Side::Covenant)
                    .face_up_defender(RoomId::Sanctum, CardName::TestMinionEndRaid),
            )
            .build();
    let id = g.client.cards.hand().find_ability_card(CardName::SariandiPhaseWalker).id();
    g.activate_ability(id, 0);
    g.click(Button::AccessSanctum);
    g.click(Button::EndRaid);
}

#[test]
fn usria() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::UsriaYinrelSpellseeker))
            .build();
    let cost_5 = g.add_to_hand(CardName::Test5CostSpell);
    assert_eq!(g.client.cards.get(cost_5).cost_icon(), "5");
    g.create_and_play(CardName::Test0CostSpell);
    assert_eq!(g.client.cards.get(cost_5).cost_icon(), "3");
    g.create_and_play(CardName::Test0CostSpell);
    assert_eq!(g.client.cards.get(cost_5).cost_icon(), "1");
    g.play_card(cost_5, g.user_id(), None);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - 1);
}
