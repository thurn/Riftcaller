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
use game_data::card_definition::Resonance;
use game_data::card_name::CardName;
use test_utils::client_interface::HasText;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::test_helpers::WeaponStats;
use test_utils::*;

#[test]
fn pathfinder() {
    let (base_attack, bonus) = (1, 2);
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .room_occupant(RoomId::RoomA, CardName::TestScheme3_10)
                .face_up_defender(RoomId::RoomA, CardName::TestInfernalMinion),
        )
        .build();
    g.create_and_play(CardName::Pathfinder);
    g.initiate_raid(RoomId::RoomA);
    assert_eq!(
        (base_attack + bonus).to_string(),
        g.client.cards.artifacts().find_card(CardName::Pathfinder).attack_icon()
    );
}

#[test]
fn pathfinder_inner_room() {
    let base_attack = 1;
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::Pathfinder);
    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(
        base_attack.to_string(),
        g.client.cards.artifacts().find_card(CardName::Pathfinder).attack_icon()
    );
}

#[test]
fn staff_of_the_valiant() {
    let stats = WeaponStats { cost: 0, attack: 1, boost_cost: 2, boost: 1 };
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .room_occupant(RoomId::RoomA, CardName::TestScheme3_10)
                .face_up_defender(RoomId::RoomA, CardName::TestInfernalMinion)
                .face_up_defender(RoomId::RoomA, CardName::TestInfernalMinion),
        )
        .build();
    g.create_and_play(CardName::StaffOfTheValiant);
    g.initiate_raid(test_constants::ROOM_ID);
    g.click_on(g.user_id(), CardName::StaffOfTheValiant.displayed_name());
    let mana = test_constants::STARTING_MANA
        - test_helpers::cost_to_play_and_defeat(stats, test_constants::MINION_HEALTH);
    assert_eq!(g.me().mana(), mana,);
    assert_eq!(
        test_constants::MINION_HEALTH.to_string(),
        g.client.cards.artifacts().find_card(CardName::StaffOfTheValiant).attack_icon()
    );

    g.click_on(g.user_id(), CardName::StaffOfTheValiant.displayed_name());
    assert_eq!(g.me().mana(), mana,);
    assert_eq!(
        test_constants::MINION_HEALTH.to_string(),
        g.client.cards.artifacts().find_card(CardName::StaffOfTheValiant).attack_icon()
    );

    g.click(Button::Score);
    assert_eq!(g.me().mana(), mana,);
    assert_eq!(
        test_constants::MINION_HEALTH.to_string(),
        g.client.cards.artifacts().find_card(CardName::StaffOfTheValiant).attack_icon()
    );
}

#[test]
fn triumph_return_to_hand() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestAstralMinion)
                .face_up_defender(RoomId::Vault, CardName::TestAstralMinion),
        )
        .build();
    g.create_and_play(CardName::Triumph);
    g.initiate_raid(RoomId::Sanctum);
    g.click_card_name(CardName::Triumph);
    g.click(Button::EndRaid);

    assert_eq!(g.client.cards.room_defenders(RoomId::Sanctum).len(), 0);
    assert!(g.opponent.cards.hand().contains_card(CardName::TestAstralMinion));

    g.initiate_raid(RoomId::Vault);
    g.click_card_name(CardName::Triumph);
    g.click(Button::EndRaid);
    assert!(g.client.cards.room_defenders(RoomId::Vault).contains_card(CardName::TestAstralMinion));
}

#[test]
fn triumph_slow() {
    let stats = WeaponStats { cost: 8, attack: 0, boost_cost: 1, boost: 1 };
    let minion_shield = 1;
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestAstralMinion1Shield),
        )
        .build();
    g.create_and_play(CardName::Triumph);
    g.initiate_raid(RoomId::Sanctum);
    g.click_card_name(CardName::Triumph);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA
            - test_helpers::cost_to_play_and_defeat(stats, test_constants::MINION_HEALTH)
            - (2 * minion_shield)
    );
}

#[test]
fn spear_of_conquest() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .actions(4)
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestAstralMinion)
                .face_up_defender(RoomId::Sanctum, CardName::TestMortalMinion2Health),
        )
        .build();
    g.create_and_play(CardName::SpearOfConquest);

    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    assert!(g
        .client
        .cards
        .artifacts()
        .find_card(CardName::SpearOfConquest)
        .arena_icon()
        .contains('1'));

    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    assert!(g
        .client
        .cards
        .artifacts()
        .find_card(CardName::SpearOfConquest)
        .arena_icon()
        .contains('2'));

    g.initiate_raid(RoomId::Sanctum);
    g.click_card_name(CardName::SpearOfConquest);
    assert!(g
        .client
        .cards
        .artifacts()
        .find_card(CardName::SpearOfConquest)
        .arena_icon()
        .contains('1'));
}

#[test]
fn spear_of_conquest_insufficient_charges() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestMortalMinion),
        )
        .build();
    g.create_and_play(CardName::SpearOfConquest);
    g.initiate_raid(RoomId::Sanctum);
    assert!(!g
        .client
        .interface
        .main_controls()
        .has_text(CardName::SpearOfConquest.displayed_name()))
}

#[test]
fn blade_of_reckoning() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .actions(5)
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestAstralMinion),
        )
        .build();
    g.create_and_play(CardName::BladeOfReckoning);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    assert!(g
        .client
        .cards
        .artifacts()
        .find_card(CardName::BladeOfReckoning)
        .arena_icon()
        .contains('3'));
    g.initiate_raid(RoomId::Sanctum);
    g.click_card_name(CardName::BladeOfReckoning);
}

#[test]
fn resolution() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::Resolution);
    g.fire_weapon_combat_abilities(Resonance::mortal(), CardName::Resolution);
    assert!(g.client.cards.discard_pile().contains_card(CardName::Resolution));
}

#[test]
fn starlight_lantern() {
    let cost = 0;
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    let id = g.create_and_play(CardName::StarlightLantern);
    g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.activate_ability(id, 1);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA - cost - test_constants::ARTIFACT_COST + 2
    );
    assert!(g.client.cards.discard_pile().contains_card(CardName::StarlightLantern));
}

#[test]
fn warriors_sign() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).actions(4).build();
    g.create_and_play(CardName::WarriorsSign);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    assert_eq!(g.me().actions(), 1);
}

#[test]
fn warriors_sign_two_of_same() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).actions(4).build();
    g.create_and_play(CardName::WarriorsSign);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::EndRaid);
    assert_eq!(g.me().actions(), 0);
}

#[test]
fn warriors_sign_alternate_order() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).actions(4).build();
    g.create_and_play_upgraded(CardName::WarriorsSign);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::EndRaid);
    assert_eq!(g.me().actions(), 2);
}

#[test]
fn chains_of_mortality() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestInfernalMinion),
        )
        .build();
    g.create_and_play(CardName::ChainsOfMortality);
    g.initiate_raid(RoomId::Sanctum);
    g.click_card_name(CardName::ChainsOfMortality);
    assert!(g.client.data.raid_active());
}

#[test]
fn chains_of_mortality_two_raids() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestInfernalMinion),
        )
        .build();
    g.create_and_play(CardName::ChainsOfMortality);
    g.initiate_raid(RoomId::Sanctum);
    assert!(g.has_card_name(CardName::ChainsOfMortality));
    g.click_card_name(CardName::ChainsOfMortality);
    g.click(Button::EndRaid);

    g.initiate_raid(RoomId::Sanctum);
    assert!(!g.has_card_name(CardName::ChainsOfMortality));
}

#[test]
fn chains_of_mortality_mortal_minion() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestMortalMinion),
        )
        .build();
    g.create_and_play(CardName::ChainsOfMortality);
    g.initiate_raid(RoomId::Sanctum);
    g.click_card_name(CardName::ChainsOfMortality);
    assert!(g.client.data.raid_active());
}

#[test]
fn phase_door() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .deck_top(CardName::TestScheme3_10)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();
    let id = g.create_and_play(CardName::PhaseDoor);
    g.activate_ability(id, 0);
    g.click(Button::Score);
}

#[test]
fn phase_door_defender() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .deck_top(CardName::TestScheme3_10)
                .face_up_defender(RoomId::Crypt, CardName::TestInfernalMinion),
        )
        .build();
    g.create_and_play(CardName::TestInfernalWeapon);
    let id = g.create_and_play(CardName::PhaseDoor);
    g.activate_ability(id, 0);
    g.click_card_name(CardName::TestInfernalWeapon);
    g.click(Button::Score);
}

#[test]
fn skyprism() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestAstralMinion),
        )
        .build();
    g.create_and_play(CardName::Skyprism);
    g.initiate_raid(RoomId::Vault);
    g.click_card_name(CardName::Skyprism);
    assert_eq!(g.me().actions(), 1);
}

#[test]
fn shield_of_the_flames() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();

    let shield_id = g.create_and_play(CardName::ShieldOfTheFlames);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(shield_id, 0);
    assert!(g.client.data.raid_active());
    assert!(g.client.cards.discard_pile().contains_card(CardName::ShieldOfTheFlames));
    g.click(Button::EndRaid);
}

#[test]
fn foebane() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestMinionShield1Infernal),
        )
        .build();
    g.create_and_play_with_target(CardName::Foebane, RoomId::Vault);
    g.click(Button::ChooseOnPlay);
    g.initiate_raid(RoomId::Vault);
    let mana = g.me().mana();
    g.click(Button::Evade);
    assert_eq!(g.me().mana(), mana - 1);
    g.click(Button::EndRaid);
}

#[test]
fn foebane_do_not_use() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestMinionShield1Infernal),
        )
        .build();
    g.create_and_play_with_target(CardName::Foebane, RoomId::Vault);
    g.click(Button::ChooseOnPlay);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::NoPromptAction);
    g.click(Button::NoWeapon);
    assert!(!g.client.data.raid_active());
}

#[test]
fn foebane_insufficient_mana() {
    let cost = 8;
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).mana(cost))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestMinionShield1Infernal),
        )
        .build();
    g.create_and_play_with_target(CardName::Foebane, RoomId::Vault);
    g.click(Button::ChooseOnPlay);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::NoWeapon);
    assert!(!g.client.data.raid_active());
}

#[test]
fn foebane_cannot_play_without_target() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    let id = g.add_to_hand(CardName::Foebane);
    assert!(g.play_card_with_result(id, g.user_id(), Some(RoomId::Vault)).is_err());
}

#[test]
fn whip_of_disjunction() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestAstralMinion),
        )
        .build();
    let id = g.create_and_play(CardName::WhipOfDisjunction);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 0);
    g.click(Button::NoWeapon);
    assert!(g.client.data.raid_active());
    g.click(Button::EndRaid);
}

#[test]
fn whip_of_disjunction_cannot_activate_mortal() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();
    let id = g.create_and_play(CardName::WhipOfDisjunction);
    g.initiate_raid(RoomId::Vault);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn whip_of_disjunction_cannot_activate_before_encounter() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_down_defender(RoomId::Vault, CardName::TestAstralMinion),
        )
        .build();
    let id = g.create_and_play(CardName::WhipOfDisjunction);
    g.initiate_raid(RoomId::Vault);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn glimmersong() {
    let cost = 3;
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestMortalMinion2Health),
        )
        .build();
    g.create_and_play(CardName::Glimmersong);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    assert_eq!(
        "2".to_string(),
        g.client.cards.artifacts().find_card(CardName::Glimmersong).attack_icon()
    );
    assert!(g.client.cards.artifacts().find_card(CardName::Glimmersong).arena_icon().contains('2'));
    g.initiate_raid(RoomId::Vault);
    g.click_card_name(CardName::Glimmersong);
    assert!(g.client.data.raid_active());
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost);
}

#[test]
fn glimmersong_does_not_trigger_on_score() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant).room_occupant(RoomId::RoomA, CardName::TestScheme3_10),
        )
        .build();
    g.create_and_play(CardName::Glimmersong);
    g.initiate_raid(RoomId::RoomA);
    g.click(Button::Score);
    g.click(Button::EndRaid);
    assert_eq!(
        "0".to_string(),
        g.client.cards.artifacts().find_card(CardName::Glimmersong).attack_icon()
    );
}

#[test]
fn glimmersong_does_not_trigger_on_raze() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .room_occupant(RoomId::RoomA, CardName::TestDuskboundProject),
        )
        .build();
    g.create_and_play(CardName::Glimmersong);
    g.initiate_raid(RoomId::RoomA);
    g.click(Button::Destroy);
    g.click(Button::EndRaid);
    assert_eq!(
        "0".to_string(),
        g.client.cards.artifacts().find_card(CardName::Glimmersong).attack_icon()
    );
}

#[test]
fn glimmersong_double_reveal() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .room_occupant(RoomId::RoomA, CardName::TestDuskboundProject),
        )
        .build();
    g.create_and_play(CardName::Glimmersong);
    // You *can* reveal the same face-down card twice
    g.create_and_play_with_target(CardName::Keensight, RoomId::RoomA);
    g.click(Button::NoPromptAction);
    g.create_and_play_with_target(CardName::Keensight, RoomId::RoomA);
    g.click(Button::NoPromptAction);
    assert_eq!(
        "2".to_string(),
        g.client.cards.artifacts().find_card(CardName::Glimmersong).attack_icon()
    );
    assert!(g.client.cards.artifacts().find_card(CardName::Glimmersong).arena_icon().contains('2'));
}

#[test]
fn glimmersong_cannot_reveal_face_up() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_room_occupant(RoomId::RoomA, CardName::TestDuskboundProject),
        )
        .build();
    g.create_and_play(CardName::Glimmersong);
    // You *cannot* reveal a face-up card.
    g.create_and_play_with_target(CardName::Keensight, RoomId::RoomA);
    g.click(Button::NoPromptAction);
    assert_eq!(
        "0".to_string(),
        g.client.cards.artifacts().find_card(CardName::Glimmersong).attack_icon()
    );
}

#[test]
fn spear_of_ultimatum() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestInfernalMinion)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();
    let id = g.create_and_play(CardName::SpearOfUltimatum);
    g.activate_ability_with_target(id, 0, RoomId::Sanctum);
    g.click(Button::ChooseForPrompt);
    g.initiate_raid(RoomId::Sanctum);
    g.click_card_name(CardName::SpearOfUltimatum);
    g.click(Button::EndRaid);

    // Cannot use on different minion
    g.initiate_raid(RoomId::Vault);
    assert!(!g.has_card_name(CardName::SpearOfUltimatum));
    g.click(Button::NoWeapon);
}

#[test]
fn spear_of_ultimatum_cannot_use_without_activating() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();
    g.create_and_play(CardName::SpearOfUltimatum);
    g.initiate_raid(RoomId::Vault);
    assert!(!g.has_card_name(CardName::SpearOfUltimatum));
}

#[test]
fn spear_of_ultimatum_cannot_target_non_infernal_minion() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestAstralMinion),
        )
        .build();
    let id = g.create_and_play(CardName::SpearOfUltimatum);
    assert!(g.activate_ability_with_target_and_result(id, 0, RoomId::Sanctum).is_err());
}

#[test]
fn spear_of_ultimatum_change_target() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Sanctum, CardName::TestInfernalMinion)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();
    let id = g.create_and_play(CardName::SpearOfUltimatum);
    g.activate_ability_with_target(id, 0, RoomId::Sanctum);
    g.click(Button::ChooseForPrompt);
    g.initiate_raid(RoomId::Sanctum);
    g.click_card_name(CardName::SpearOfUltimatum);
    g.click(Button::EndRaid);

    g.activate_ability_with_target(id, 0, RoomId::Vault);
    g.click(Button::ChooseForPrompt);
    g.initiate_raid(RoomId::Vault);
    g.click_card_name(CardName::SpearOfUltimatum);
    g.click(Button::EndRaid);
}

#[test]
fn maul_of_devastation() {
    let (cost, shield, base_attack) = (0, 3, 1);
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestMinionShield3Mortal),
        )
        .build();
    g.create_and_play(CardName::MaulOfDevastation);
    g.initiate_raid(RoomId::Vault);
    g.click_card_name(CardName::MaulOfDevastation);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA
            - cost
            - (shield * 2)
            - (test_constants::MINION_HEALTH - base_attack)
    );
    g.click(Button::EndRaid);
}

#[test]
fn maul_of_devastation_after_access() {
    let (cost, shield, base_attack) = (0, 3, 1);
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestMinionShield3Mortal),
        )
        .build();
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.create_and_play(CardName::MaulOfDevastation);
    g.initiate_raid(RoomId::Vault);
    g.click_card_name(CardName::MaulOfDevastation);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA
            - cost
            - shield
            - (test_constants::MINION_HEALTH - base_attack)
    );
    g.click(Button::EndRaid);
}

#[test]
fn amaras_decree() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.pass_turn(Side::Covenant);
    g.create_and_play(CardName::AmarasDecree);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    assert_eq!(g.me().score(), 0);
    g.pass_turn(Side::Covenant);
    g.pass_turn(Side::Riftcaller);
    assert_eq!(g.me().score(), 10);
}

#[test]
fn amaras_decree_destroyed() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.pass_turn(Side::Covenant);
    g.create_and_play(CardName::AmarasDecree);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    assert_eq!(g.me().score(), 0);
    g.create_and_play(CardName::TestRitualDestroyAllEnemyPermanents);
    assert_eq!(g.me().score(), 10);
}
