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
use game_data::card_name::{CardName, CardVariant};
use test_utils::test_game::{TestGame, TestSide};
use test_utils::test_session::TestSession;
use test_utils::*;

#[test]
fn restoration() {
    let cost = 1;
    let mut g =
        TestGame::new(TestSide::new(Side::Champion).in_discard_face_up(CardName::TestAstralWeapon))
            .build();
    assert!(g.client.cards.artifacts().is_empty());
    g.create_and_play(CardName::Restoration);
    test_helpers::assert_cards_match(g.client.cards.hand(), vec![CardName::TestAstralWeapon]);
    let id = g.client.cards.hand().find_card_id(CardName::TestAstralWeapon);
    g.play_card(id, g.user_id(), None);
    assert!(g.client.cards.hand().is_empty());
    test_helpers::assert_cards_match(g.client.cards.artifacts(), vec![CardName::TestAstralWeapon]);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost - test_constants::WEAPON_COST);
}

#[test]
fn restoration_no_targets() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(3)).build();
    assert_eq!(g.client.cards.hand().len(), 3);
    let id = g.add_to_hand(CardName::Restoration);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
}

#[test]
fn restoration_last_action_point() {
    let mut g =
        TestGame::new(TestSide::new(Side::Champion).in_discard_face_up(CardName::TestAstralWeapon))
            .actions(1)
            .build();
    g.create_and_play(CardName::Restoration);
    test_helpers::assert_cards_match(g.client.cards.hand(), vec![CardName::TestAstralWeapon]);
    let id = g.client.cards.hand().find_card_id(CardName::TestAstralWeapon);
    g.play_card(id, g.user_id(), None);
    assert!(g.has(Button::EndTurn));
}

#[test]
fn restoration_cannot_take_other_action() {
    let mut g =
        TestGame::new(TestSide::new(Side::Champion).in_discard_face_up(CardName::TestAstralWeapon))
            .build();
    assert!(g.client.cards.artifacts().is_empty());
    g.create_and_play(CardName::Restoration);
    test_helpers::assert_cards_match(g.client.cards.hand(), vec![CardName::TestAstralWeapon]);
    assert!(g.draw_card_with_result().is_err());
}

#[test]
fn restoration_upgraded() {
    let (cost, reduction) = (1, 2);
    let mut g =
        TestGame::new(TestSide::new(Side::Champion).in_discard_face_up(CardName::TestAstralWeapon))
            .build();
    assert!(g.client.cards.artifacts().is_empty());
    g.create_and_play_upgraded(CardName::Restoration);
    test_helpers::assert_cards_match(g.client.cards.hand(), vec![CardName::TestAstralWeapon]);
    let id = g.client.cards.hand().find_card_id(CardName::TestAstralWeapon);
    g.play_card(id, g.user_id(), None);
    assert!(g.client.cards.hand().is_empty());
    test_helpers::assert_cards_match(g.client.cards.artifacts(), vec![CardName::TestAstralWeapon]);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA - cost - test_constants::WEAPON_COST + reduction
    );
}

#[test]
fn restoration_upgraded_stacking() {
    let (cost, reduction) = (1, 2);
    let mut g = TestGame::new(
        TestSide::new(Side::Champion)
            .in_discard_face_up(CardName::TestWeaponReduceCostOnSuccessfulRaid),
    )
    .build();
    assert!(g.client.cards.artifacts().is_empty());
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.create_and_play_upgraded(CardName::Restoration);
    let id = g.client.cards.hand().find_card_id(CardName::TestWeaponReduceCostOnSuccessfulRaid);
    g.play_card(id, g.user_id(), None);
    assert!(g.client.cards.hand().is_empty());
    // Test weapon costs 5 and reduces cost by 2 on raid access
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost - 5 + reduction + 2);
}

#[test]
fn strike_the_heart() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(TestSide::new(Side::Overlord).hand_size(5))
        .build();
    g.create_and_play(CardName::StrikeTheHeart);
    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 3);
}

#[test]
fn strike_the_heart_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(TestSide::new(Side::Overlord).hand_size(5))
        .build();
    g.create_and_play_upgraded(CardName::StrikeTheHeart);
    assert_eq!(g.client.cards.browser().iter().filter(|c| c.revealed_to_me()).count(), 4);
}

#[test]
fn enduring_radiance() {
    let (cost, return_cost) = (0, 1);
    let mut g = TestGame::new(TestSide::new(Side::Champion).curses(2)).build();
    assert_eq!(g.client.cards.hand().curse_count(), 2);
    g.create_and_play(CardName::EnduringRadiance);
    assert_eq!(g.client.cards.hand().curse_count(), 1);
    g.click(Button::ReturnToHand);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost - return_cost);
    let id = g.client.cards.hand().find_card_id(CardName::EnduringRadiance);
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.client.cards.hand().curse_count(), 0);
}

#[test]
fn enduring_radiance_no_curses() {
    let (cost, return_cost) = (0, 1);
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::EnduringRadiance);
    assert_eq!(g.client.cards.hand().curse_count(), 0);
    g.click(Button::ReturnToHand);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost - return_cost);
    let id = g.client.cards.hand().find_card_id(CardName::EnduringRadiance);
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.client.cards.hand().curse_count(), 0);
}

#[test]
fn enduring_radiance_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).curses(2)).build();
    assert_eq!(g.client.cards.hand().curse_count(), 2);
    g.create_and_play_upgraded(CardName::EnduringRadiance);
    assert_eq!(g.client.cards.hand().curse_count(), 1);
    let id = g.client.cards.hand().find_card_id(CardName::EnduringRadiance);
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.client.cards.hand().curse_count(), 0);
}

#[test]
fn sift_the_sands() {
    let (cost, reduction) = (1, 3);
    let mut g =
        TestGame::new(TestSide::new(Side::Champion).deck_top(CardName::TestEvocation)).build();
    g.create_and_play(CardName::SiftTheSands);
    assert_eq!(g.client.cards.hand().len(), 4);
    let id = g.client.cards.find_in_hand(CardVariant::standard(CardName::TestEvocation));
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.client.cards.discard_pile().len(), 4);
    test_helpers::assert_cards_match(
        g.client.cards.evocations_and_allies(),
        vec![CardName::TestEvocation],
    );
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA - cost - (test_constants::EVOCATION_COST - reduction)
    )
}

#[test]
fn sift_the_sands_upgraded() {
    let mut g =
        TestGame::new(TestSide::new(Side::Champion).deck_top(CardName::TestEvocation)).build();
    g.create_and_play_upgraded(CardName::SiftTheSands);
    assert_eq!(g.client.cards.hand().len(), 6);
    let id = g.client.cards.find_in_hand(CardVariant::standard(CardName::TestEvocation));
    g.play_card(id, g.user_id(), None);
}

#[test]
fn holy_aura() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::HolyAura);
    assert_eq!(g.client.cards.hand().len(), 3);
}

#[test]
fn holy_aura_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play_upgraded(CardName::HolyAura);
    assert_eq!(g.client.cards.hand().len(), 4);
}

#[test]
fn holy_aura_discard_to_damage() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.add_to_hand(CardName::HolyAura);
    g.pass_turn(Side::Champion);
    assert_eq!(g.client.cards.hand().len(), 1);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    assert_eq!(g.client.cards.hand().len(), 2);
    test_helpers::assert_cards_match(g.client.cards.discard_pile(), vec![CardName::HolyAura]);
}

#[test]
fn holy_aura_discard_to_hand_size() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(5)).build();
    let id = g.add_to_hand(CardName::HolyAura);
    g.move_to_end_step(Side::Champion);
    g.move_selector_card(id);
    g.click(Button::SubmitDiscard);
    assert_eq!(g.client.cards.hand().len(), 7);
    test_helpers::assert_cards_match(g.client.cards.discard_pile(), vec![CardName::HolyAura]);
}

#[test]
fn holy_aura_discard_to_sift_the_sands() {
    let mut g = TestGame::new(
        TestSide::new(Side::Champion)
            .deck_top(CardName::HolyAura)
            .deck_top(CardName::TestEvocation),
    )
    .build();
    g.create_and_play(CardName::SiftTheSands);
    let id = g.client.cards.find_in_hand(CardVariant::standard(CardName::TestEvocation));
    g.play_card(id, g.user_id(), None);
    assert_eq!(g.client.cards.hand().len(), 2);
}

#[test]
fn voidstep() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();
    g.create_and_play_with_target(CardName::Voidstep, RoomId::Vault);
    assert!(g.has(Button::EndRaid));
    assert!(g.client.data.raid_active());
}

#[test]
fn voidstep_two_defenders() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion)
                .face_up_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();
    g.create_and_play_with_target(CardName::Voidstep, RoomId::Vault);
    assert_eq!(g.client.cards.raid_display().len(), 1);
}

#[test]
fn keensight() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord).room_occupant(RoomId::RoomA, CardName::TestScheme3_10),
        )
        .build();
    assert!(!g.client.cards.room_occupants(RoomId::RoomA)[0].revealed_to_me());
    g.create_and_play_with_target(CardName::Keensight, RoomId::RoomA);
    assert!(g.client.cards.room_occupants(RoomId::RoomA)[0].revealed_to_me());
    g.click(Button::InitiateRaid);
    assert!(g.client.data.raid_active());
}

#[test]
fn ethereal_incursion() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_down_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();
    g.create_and_play_with_target(CardName::EtherealIncursion, RoomId::Vault);
    g.click_as_side(Button::Summon, Side::Overlord);
    g.click(Button::NoWeapon);
    assert!(!g.client.data.raid_active());
    assert!(!g.client.cards.room_defenders(RoomId::Vault)[0].is_face_up())
}

#[test]
fn time_stop() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play_with_target(CardName::TimeStop, RoomId::Vault);
    g.click(Button::EndRaid);
    assert_eq!(g.me().actions(), 4);
}

#[test]
fn time_stop_cannot_play_second() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.gain_mana();
    let id = g.add_to_hand(CardName::TimeStop);
    assert!(g.play_card_with_result(id, g.user_id(), Some(RoomId::Vault)).is_err());
}

#[test]
fn chains_of_binding() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_down_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();
    g.create_and_play_with_target(CardName::ChainsOfBinding, RoomId::Vault);
    g.click(Button::ChooseDefenderForPrompt);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::EndRaid);
}

#[test]
fn chains_of_binding_multiple_turns() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_down_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();
    g.create_and_play_with_target(CardName::ChainsOfBinding, RoomId::Vault);
    g.click(Button::ChooseDefenderForPrompt);
    g.pass_turn(Side::Champion);
    g.pass_turn(Side::Overlord);
    g.initiate_raid(RoomId::Vault);
    g.opponent_click(Button::Summon);
    g.click(Button::NoWeapon);
}

#[test]
fn chains_of_binding_duskbound_project() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .room_occupant(RoomId::RoomA, CardName::TestProjectTriggeredAbilityTakeManaAtDusk),
        )
        .build();
    g.create_and_play_with_target(CardName::ChainsOfBinding, RoomId::RoomA);
    g.click(Button::ChooseOccupantForPrompt);
    g.move_to_end_step(Side::Champion);
    assert!(g.dusk());
}

fn raid_inner_rooms(g: &mut TestSession) {
    g.initiate_raid(RoomId::Vault);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
}

#[test]
fn delve_into_darkness() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .actions(5)
        .opponent(
            TestSide::new(Side::Overlord)
                .deck_top(CardName::TestScheme3_10)
                .deck_top(CardName::TestDuskboundProject),
        )
        .build();
    raid_inner_rooms(&mut g);

    g.create_and_play(CardName::DelveIntoDarkness);
    assert!(g.client.data.raid_active());
    g.click(Button::Score);
    assert_eq!(g.me().actions(), 1);
    g.click(Button::AccessAnother);
    assert_eq!(g.me().actions(), 0);
    g.click(Button::Discard);
    assert!(!g.client.data.raid_active());
    g.click(Button::EndTurn);
    assert_eq!(g.me().score(), 10);
    assert!(g.client.cards.opponent_discard_pile().contains_card(CardName::TestDuskboundProject));
}

#[test]
fn delve_into_darkness_end_access_after_0() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .actions(5)
        .opponent(
            TestSide::new(Side::Overlord)
                .deck_top(CardName::TestScheme3_10)
                .deck_top(CardName::TestDuskboundProject),
        )
        .build();
    raid_inner_rooms(&mut g);

    g.create_and_play(CardName::DelveIntoDarkness);
    assert!(g.client.data.raid_active());
    g.click(Button::EndAccess);
    assert!(!g.client.data.raid_active());
}

#[test]
fn delve_into_darkness_end_access_after_1() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .actions(5)
        .opponent(
            TestSide::new(Side::Overlord)
                .deck_top(CardName::TestScheme3_10)
                .deck_top(CardName::TestDuskboundProject),
        )
        .build();
    raid_inner_rooms(&mut g);

    g.create_and_play(CardName::DelveIntoDarkness);
    g.click(Button::Score);
    assert_eq!(g.me().actions(), 1);
    g.click(Button::EndAccess);
    assert!(!g.client.data.raid_active());
    assert_eq!(g.me().score(), 10);
}

#[test]
fn delve_into_darkness_cannot_play_failed_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Vault, CardName::TestMinionEndRaid),
        )
        .build();
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::NoWeapon);

    let id = g.add_to_hand(CardName::DelveIntoDarkness);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
}

#[test]
fn delve_into_darkness_does_not_count_for_glimmersong() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .actions(5)
        .opponent(
            TestSide::new(Side::Overlord)
                .deck_top(CardName::TestScheme3_10)
                .deck_top(CardName::TestDuskboundProject),
        )
        .build();
    raid_inner_rooms(&mut g);
    let glimmersong = g.create_and_play(CardName::Glimmersong);
    g.create_and_play(CardName::DelveIntoDarkness);
    g.click(Button::EndAccess);
    assert_eq!(g.client.cards.get(glimmersong).attack_icon(), "0")
}

#[test]
fn liminal_transposition() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .room_occupant(RoomId::RoomA, CardName::TestProject2Cost3Raze),
        )
        .build();
    g.create_and_play_with_target(CardName::LiminalTransposition, RoomId::Sanctum);
    let room_selector = g.client.cards.hand().find_card_id(CardName::LiminalTransposition);
    g.play_card(room_selector, g.user_id(), Some(RoomId::RoomA));
    g.click(Button::Destroy);
    g.click(Button::EndRaid);
    assert!(g.client.cards.opponent_discard_pile().contains_card(CardName::TestProject2Cost3Raze));
}

#[test]
fn liminal_transposition_cannot_score() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord).room_occupant(RoomId::RoomA, CardName::TestScheme3_10),
        )
        .build();
    g.create_and_play_with_target(CardName::LiminalTransposition, RoomId::Sanctum);
    let room_selector = g.client.cards.hand().find_card_id(CardName::LiminalTransposition);
    g.play_card(room_selector, g.user_id(), Some(RoomId::RoomA));
    assert!(!g.has(Button::Score));
}

#[test]
fn liminal_transposition_cannot_target_single_outer_room() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord).room_occupant(RoomId::RoomA, CardName::TestScheme3_10),
        )
        .build();
    let id = g.add_to_hand(CardName::LiminalTransposition);
    assert!(g.play_card_with_result(id, g.user_id(), Some(RoomId::RoomA)).is_err());
}

#[test]
fn liminal_transposition_can_target_outer_room_with_2_occupied() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .room_occupant(RoomId::RoomA, CardName::TestScheme3_10)
                .room_occupant(RoomId::RoomB, CardName::TestProject2Cost3Raze),
        )
        .build();
    g.create_and_play_with_target(CardName::LiminalTransposition, RoomId::RoomA);
    let room_selector = g.client.cards.hand().find_card_id(CardName::LiminalTransposition);
    g.play_card(room_selector, g.user_id(), Some(RoomId::RoomB));
    g.click(Button::Destroy);
}

#[test]
fn liminal_transposition_cannot_target_same_room() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .room_occupant(RoomId::RoomA, CardName::TestScheme3_10)
                .room_occupant(RoomId::RoomB, CardName::TestProject2Cost3Raze),
        )
        .build();
    g.create_and_play_with_target(CardName::LiminalTransposition, RoomId::RoomA);
    let room_selector = g.client.cards.hand().find_card_id(CardName::LiminalTransposition);
    assert!(g.play_card_with_result(room_selector, g.user_id(), Some(RoomId::RoomA)).is_err());
}

#[test]
fn liminal_transposition_counts_for_warriors_sign() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord).room_occupant(RoomId::RoomA, CardName::TestScheme3_10),
        )
        .actions(4)
        .build();
    g.create_and_play(CardName::WarriorsSign);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Vault);
    g.click(Button::EndRaid);
    g.create_and_play_with_target(CardName::LiminalTransposition, RoomId::Crypt);
    let room_selector = g.client.cards.hand().find_card_id(CardName::LiminalTransposition);
    g.play_card(room_selector, g.user_id(), Some(RoomId::RoomA));
    g.click(Button::EndRaid);
    assert_eq!(g.me().actions(), 1);
}

#[test]
fn liminal_transposition_counts_for_glimmersong() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord).room_occupant(RoomId::RoomA, CardName::TestScheme3_10),
        )
        .build();
    let glimmersong = g.create_and_play(CardName::Glimmersong);
    g.create_and_play_with_target(CardName::LiminalTransposition, RoomId::Sanctum);
    let room_selector = g.client.cards.hand().find_card_id(CardName::LiminalTransposition);
    g.play_card(room_selector, g.user_id(), Some(RoomId::RoomA));
    g.click(Button::EndRaid);
    assert!(g.client.cards.get(glimmersong).arena_icon().contains('1'));
}
