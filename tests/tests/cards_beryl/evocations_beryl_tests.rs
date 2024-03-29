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
use protos::riftcaller::object_position::Position;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn empyreal_chorus() {
    let (cost, gained) = (1, 8);
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .room_occupant(test_constants::ROOM_ID, CardName::TestScheme3_10),
        )
        .build();
    let id = g.create_and_play(CardName::EmpyrealChorus);
    g.activate_ability_with_target(id, 0, test_constants::ROOM_ID);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost + gained);
    assert!(!g.client.data.raid_active());
    assert!(!g.client.cards.room_occupants(test_constants::ROOM_ID)[0].revealed_to_me());
}

#[test]
fn starfield_omen() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::StarfieldOmen);
    let id = g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    assert_eq!(g.client.cards.hand().real_cards().len(), 0);
    g.activate_ability(id, 0);
    assert!(g.client.cards.discard_pile().contains_card(CardName::TestSacrificeDrawCardArtifact));
    assert_eq!(g.client.cards.hand().real_cards().len(), 2);
}

#[test]
fn visitation() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).hand_size(1)).build();
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDeal1Damage);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.hand().len(), 1);
    assert!(g.client.cards.discard_pile().contains_card(CardName::Visitation));
}

#[test]
fn visitation_pass() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).hand_size(1)).build();
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDeal1Damage);
    g.click(Button::NoPromptAction);
    assert_eq!(g.client.cards.hand().len(), 0);
    assert!(g.client.cards.evocations_and_allies().contains_card(CardName::Visitation));
}

#[test]
fn visitation_multiple_copies() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).hand_size(1)).build();
    g.create_and_play(CardName::Visitation);
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDeal1Damage);
    g.click(Button::NoPromptAction);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.hand().len(), 1);
    assert!(g.client.cards.discard_pile().contains_card(CardName::Visitation));
    assert!(g.client.cards.evocations_and_allies().contains_card(CardName::Visitation));
}

#[test]
fn visitation_prevent_partial() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).hand_size(5)).build();
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDeal5Damage);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.hand().len(), 2);
}

#[test]
fn visitation_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).hand_size(5)).build();
    g.create_and_play_upgraded(CardName::Visitation);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDeal5Damage);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.hand().len(), 5);
}

#[test]
fn backup_plan() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();
    let id = g.create_and_play(CardName::BackupPlan);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 0);
    assert!(g.client.data.raid_active());
    g.click(Button::EndRaid);
    assert_eq!(g.me().actions(), 0);
}

#[test]
fn backup_plan_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .actions(4)
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();
    let id = g.create_and_play_upgraded(CardName::BackupPlan);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 0);
    assert_eq!(g.me().actions(), 1);
}

#[test]
fn backup_plan_cannot_activate_outside_encounter() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(TestSide::new(Side::Covenant))
        .build();
    let id = g.create_and_play(CardName::BackupPlan);
    g.initiate_raid(RoomId::Vault);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn planar_sanctuary() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant).room_occupant(RoomId::RoomA, CardName::TestScheme3_10),
        )
        .build();
    let id = g.create_and_play(CardName::PlanarSanctuary);
    g.initiate_raid(RoomId::RoomA);
    g.click(Button::Score);
    g.activate_ability(id, 1);
    assert_eq!(g.client.cards.hand().real_cards().len(), 1);
    assert!(g
        .client
        .cards
        .evocations_and_allies()
        .find_card(CardName::PlanarSanctuary)
        .arena_icon()
        .contains('1'));
    g.click(Button::EndRaid);
}

#[test]
fn planar_sanctuary_activate_after_curse() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    let id = g.create_and_play(CardName::PlanarSanctuary);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    assert!(g
        .client
        .cards
        .evocations_and_allies()
        .find_card(CardName::PlanarSanctuary)
        .arena_icon()
        .contains('2'));
    g.create_and_play(CardName::TestRitualGiveCurse);
    assert!(g.me().can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
    assert_eq!(g.client.cards.hand().curse_count(), 1);
    assert_eq!(g.client.cards.hand().real_cards().len(), 0);
    g.activate_ability(id, 1);
    assert_eq!(g.client.cards.hand().curse_count(), 0);
    assert_eq!(g.client.cards.hand().real_cards().len(), 1);
    assert!(g.me().can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
    assert!(g
        .client
        .cards
        .evocations_and_allies()
        .find_card(CardName::PlanarSanctuary)
        .arena_icon()
        .contains('1'));
    g.click(Button::ClosePriorityPrompt);
    assert!(!g.me().can_take_action());
    assert!(g.opponent.this_player.can_take_action());
}

#[test]
fn planar_sanctuary_activate_after_damage() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).hand_size(5)).build();
    let id = g.create_and_play(CardName::PlanarSanctuary);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    g.create_and_play(CardName::TestRitualDeal1Damage);
    assert_eq!(g.client.cards.hand().real_cards().len(), 4);
    assert!(g.me().can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
    g.activate_ability(id, 1);
    assert_eq!(g.client.cards.hand().real_cards().len(), 5);
    assert!(g.me().can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
    g.click(Button::ClosePriorityPrompt);
    assert!(!g.me().can_take_action());
    assert!(g.opponent.this_player.can_take_action());
}

#[test]
fn knowledge_of_the_beyond() {
    let (cost, reduction) = (0, 1);
    let mut g = TestGame::new(
        TestSide::new(Side::Riftcaller)
            .deck_top(CardName::TestSacrificeDrawCardArtifact)
            .deck_top(CardName::TestEvocation),
    )
    .build();

    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.activate_ability(id, 1);
    let artifact_id = g.client.cards.hand().find_card_id(CardName::TestSacrificeDrawCardArtifact);
    g.play_card(artifact_id, g.user_id(), None);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA - cost - (test_constants::ARTIFACT_COST - reduction)
    );
    assert!(g.client.cards.discard_pile().contains_card(CardName::KnowledgeOfTheBeyond));
    assert!(g.client.cards.discard_pile().contains_card(CardName::TestEvocation));
    assert!(g.client.cards.artifacts().contains_card(CardName::TestSacrificeDrawCardArtifact));
}

#[test]
fn knowledge_of_the_beyond_activate_for_weapon_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).deck_top(CardName::TestMortalWeapon))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();

    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 1);
    let artifact_id = g.client.cards.hand().find_card_id(CardName::TestMortalWeapon);
    g.play_card(artifact_id, g.user_id(), None);
    g.click_card_name(CardName::TestMortalWeapon);
    assert!(g.client.data.raid_active());
    g.click(Button::EndRaid);
}

#[test]
fn knowledge_of_the_beyond_activate_during_access() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).deck_top(CardName::TestMortalWeapon))
        .opponent(TestSide::new(Side::Covenant).deck_top(CardName::TestScheme3_10))
        .build();

    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 1);
    let artifact_id = g.client.cards.hand().find_card_id(CardName::TestMortalWeapon);
    g.play_card(artifact_id, g.user_id(), None);
    assert!(g.client.cards.artifacts().contains_card(CardName::TestMortalWeapon));
    g.click(Button::Score);
}

#[test]
fn knowledge_of_the_beyond_no_hits() {
    let mut g = TestGame::new(
        TestSide::new(Side::Riftcaller)
            .deck_top(CardName::TestSpell)
            .deck_top(CardName::TestSpell)
            .deck_top(CardName::TestSpell)
            .hand_size(5),
    )
    .build();

    // You can still activate it with no hits to e.g. put the cards into your
    // discard pile. No prompt is shown.
    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.activate_ability(id, 1);
    assert_eq!(g.client.cards.hand().len(), 5);
    assert_eq!(g.client.cards.discard_pile().len(), 4);
}

#[test]
fn knowledge_of_the_beyond_activate_planar_sanctuary() {
    let mut g = TestGame::new(
        TestSide::new(Side::Riftcaller).deck_top(CardName::TestMortalWeapon).hand_size(5),
    )
    .build();
    let knowledge_of_the_beyond = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.create_and_play(CardName::PlanarSanctuary);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    g.create_and_play(CardName::TestRitualDeal1Damage);
    g.activate_ability(knowledge_of_the_beyond, 1);
    let artifact_id = g.client.cards.hand().find_card_id(CardName::TestMortalWeapon);
    g.play_card(artifact_id, g.user_id(), None);
    assert!(g.client.cards.artifacts().contains_card(CardName::TestMortalWeapon));
    assert!(g.client.cards.artifacts().find_card(CardName::TestMortalWeapon).is_face_up());
    g.click(Button::ClosePriorityPrompt);
}

#[test]
fn knowledge_of_the_beyond_activate_planar_sanctuary_first() {
    let mut g = TestGame::new(
        TestSide::new(Side::Riftcaller).deck_top(CardName::TestMortalWeapon).hand_size(5),
    )
    .build();
    let knowledge_of_the_beyond = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    let planar_sanctuary = g.create_and_play(CardName::PlanarSanctuary);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    g.create_and_play(CardName::TestRitualDeal1Damage);
    g.activate_ability(planar_sanctuary, 1);
    g.activate_ability(knowledge_of_the_beyond, 1);
    let artifact_id = g.client.cards.hand().find_card_id(CardName::TestMortalWeapon);
    g.play_card(artifact_id, g.user_id(), None);
    assert!(g.client.cards.artifacts().contains_card(CardName::TestMortalWeapon));
    assert!(g.client.cards.artifacts().find_card(CardName::TestMortalWeapon).is_face_up());
    g.click(Button::ClosePriorityPrompt);
}

#[test]
fn knowledge_of_the_beyond_card_target_with_foebane() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).deck_top(CardName::Foebane))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_up_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();

    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 1);
    let foebane = g.client.cards.hand().find_card_id(CardName::Foebane);
    g.play_card(foebane, g.user_id(), Some(RoomId::Vault));
    g.click(Button::ChooseOnPlay);

    // This is a bit weird, but basically Foebane triggers during the approach step,
    // which is what allows it to by pass "on encounter" abilities, so you can't put
    // it into play and then use it immediately during an encounter. We could prompt
    // *again* to use it on encounter but that would be very annoying.
    assert!(!g.has(Button::Evade));
}

#[test]
fn knowledge_of_the_beyond_shield_of_the_flames_evade() {
    let mut g = TestGame::new(
        TestSide::new(Side::Riftcaller)
            .deck_top(CardName::ShieldOfTheFlames)
            .deck_top(CardName::TestSacrificeDrawCardArtifact),
    )
    .opponent(
        TestSide::new(Side::Covenant).face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
    )
    .build();

    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 1);
    let shield = g.client.cards.hand().find_card_id(CardName::ShieldOfTheFlames);
    g.play_card(shield, g.user_id(), None);

    // Unlike with Foebane you *can* evade a minion with Shield of the Flames
    // because this ability happens *during* an encounter and does not bypass
    // encounter triggers.
    g.activate_ability(shield, 0);
    assert!(g.client.cards.discard_pile().contains_card(CardName::ShieldOfTheFlames));
    assert!(g.client.cards.discard_pile().contains_card(CardName::KnowledgeOfTheBeyond));
    assert!(g.client.cards.discard_pile().contains_card(CardName::TestSacrificeDrawCardArtifact));
    assert!(g.client.data.raid_active());
    g.click(Button::EndRaid);
}

#[test]
fn splinter_of_twilight_play_for_free() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.add_to_hand(CardName::SplinterOfTwilight);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::Play);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA);
    assert!(g.client.cards.evocations_and_allies().contains_card(CardName::SplinterOfTwilight));
}

#[test]
fn splinter_of_twilight_access_crypt() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(TestSide::new(Side::Covenant).in_discard_face_down(CardName::TestScheme3_10))
        .build();
    let id = g.create_and_play(CardName::SplinterOfTwilight);
    g.activate_ability(id, 1);
    g.click(Button::Score);
    assert_eq!(g.me().score(), 10);
    assert!(g.client.cards.discard_pile().contains_card(CardName::SplinterOfTwilight));
}

#[test]
fn splinter_of_twilight_play_from_phase_door() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.add_to_hand(CardName::SplinterOfTwilight);
    let id = g.create_and_play(CardName::PhaseDoor);
    g.activate_ability(id, 0);
    g.click(Button::Play);
    assert!(g.client.cards.evocations_and_allies().contains_card(CardName::SplinterOfTwilight));
}

#[test]
fn a_moments_peace() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(TestSide::new(Side::Covenant).bonus_points(50))
        .build();
    let id = g.create_and_play(CardName::AMomentsPeace);
    g.pass_turn(Side::Riftcaller);
    g.pass_turn(Side::Covenant);
    assert!(g.client.cards.get(id).arena_icon().contains('1'));
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    assert_eq!(g.client.other_player.score(), 60);
    g.pass_turn(Side::Covenant);
    assert!(g.client.cards.get(id).arena_icon().contains('2'));
    g.pass_turn(Side::Riftcaller);
    g.pass_turn(Side::Covenant);
    assert!(g.is_victory_for_player(Side::Covenant));
}

#[test]
fn a_moments_peace_return_to_hand() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(TestSide::new(Side::Covenant).bonus_points(50))
        .build();
    g.create_and_play(CardName::AMomentsPeace);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    assert_eq!(g.client.other_player.score(), 60);
    g.pass_turn(Side::Covenant);
    g.create_and_play(CardName::TestSpellReturnAllYourPermanentsToHand);
    assert!(g.is_victory_for_player(Side::Covenant));
}

#[test]
fn vortex_portal() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_down_defender(RoomId::Sanctum, CardName::TestMinionEndRaid),
        )
        .build();
    let id = g.create_and_play(CardName::VortexPortal);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    assert!(g.client.cards.get(id).arena_icon().contains('1'));
    g.pass_turn(Side::Covenant);
    g.activate_ability(id, 1);
    assert_eq!(g.client.cards.browser().len(), 1);
    g.click(Button::EndRaid);
}

#[test]
fn vortex_portal_sentinel_sphinx() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_down_defender(RoomId::Sanctum, CardName::SentinelSphinx),
        )
        .build();
    let id = g.create_and_play(CardName::VortexPortal);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    g.pass_turn(Side::Covenant);
    g.activate_ability(id, 1);
    g.opponent_click(Button::Summon);
    g.click(Button::NoWeapon);
    assert!(!g.client.data.raid_active());
}

#[test]
fn radiant_intervention() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.create_and_play(CardName::RadiantIntervention);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDestroyAllEnemyPermanents);
    g.click(Button::Prevent);
    assert!(g.client.cards.artifacts().contains_card(CardName::TestSacrificeDrawCardArtifact));
    assert!(g.client.cards.discard_pile().contains_card(CardName::RadiantIntervention));
}

#[test]
fn radiant_intervention_do_not_use() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.create_and_play(CardName::RadiantIntervention);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDestroyAllEnemyPermanents);
    g.click(Button::NoPromptAction);
    assert!(g.client.cards.discard_pile().contains_card(CardName::TestSacrificeDrawCardArtifact));
}

#[test]
fn radiant_intervention_two_cards() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.create_and_play(CardName::TestWeapon2Attack);
    g.create_and_play(CardName::RadiantIntervention);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDestroyAllEnemyPermanents);
    g.click(Button::Prevent);
    assert!(g.client.cards.artifacts().contains_card(CardName::TestWeapon2Attack));
    assert!(g.client.cards.discard_pile().contains_card(CardName::RadiantIntervention));
    assert!(g.client.cards.discard_pile().contains_card(CardName::TestSacrificeDrawCardArtifact));
}

#[test]
fn radiant_intervention_multiple_copies() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.create_and_play(CardName::RadiantIntervention);
    g.create_and_play(CardName::RadiantIntervention);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDestroyAllEnemyPermanents);
    g.click(Button::Prevent);
    assert!(g.client.cards.artifacts().contains_card(CardName::TestSacrificeDrawCardArtifact));
    assert!(!g.has(Button::Prevent))
}

#[test]
fn radiant_intervention_multiple_copies_use_second() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.create_and_play(CardName::RadiantIntervention);
    g.create_and_play(CardName::RadiantIntervention);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDestroyAllEnemyPermanents);
    g.click(Button::NoPromptAction);
    g.click(Button::Prevent);
    assert!(g.client.cards.artifacts().contains_card(CardName::TestSacrificeDrawCardArtifact));
}

#[test]
fn radiant_intervention_multiple_copies_prevent_multiple() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.create_and_play(CardName::TestMortalWeapon);
    g.create_and_play(CardName::RadiantIntervention);
    g.create_and_play(CardName::RadiantIntervention);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDestroyAllEnemyPermanents);
    g.click(Button::Prevent);
    g.click(Button::Prevent);
    assert!(g.client.cards.artifacts().contains_card(CardName::TestSacrificeDrawCardArtifact));
    assert!(g.client.cards.artifacts().contains_card(CardName::TestMortalWeapon));
}

#[test]
fn lightcallers_command() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_down_defender(RoomId::Vault, CardName::TestMinionEndRaid),
        )
        .build();
    let id = g.create_and_play(CardName::LightcallersCommand);
    g.activate_ability(id, 0);
    g.initiate_raid(RoomId::Vault);
    assert!(g.has(Button::EndRaid));
}

#[test]
fn lightcallers_command_two_defenders() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_down_defender(RoomId::Vault, CardName::TestMinionDealDamage)
                .face_down_defender(RoomId::Vault, CardName::TestMinionEndRaid),
        )
        .build();
    let id = g.create_and_play(CardName::LightcallersCommand);
    g.activate_ability(id, 0);
    g.initiate_raid(RoomId::Vault);
    g.opponent_click(Button::Summon);
    g.click(Button::NoWeapon);
    assert!(g.is_victory_for_player(Side::Covenant));
}

#[test]
fn lightcallers_command_two_raids() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_down_defender(RoomId::Vault, CardName::TestMinionEndRaid)
                .face_down_defender(RoomId::Sanctum, CardName::TestMinionEndRaid),
        )
        .build();
    let id = g.create_and_play(CardName::LightcallersCommand);
    g.activate_ability(id, 0);
    g.initiate_raid(RoomId::Vault);
    assert!(g.has(Button::EndRaid));
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Sanctum);
    assert!(g.has(Button::EndRaid));
    g.click(Button::EndRaid);
}

#[test]
fn lightcallers_command_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller))
        .opponent(
            TestSide::new(Side::Covenant)
                .face_down_defender(RoomId::Vault, CardName::TestMinionDealDamage)
                .face_down_defender(RoomId::Vault, CardName::TestMinionEndRaid),
        )
        .build();
    let id = g.create_and_play_upgraded(CardName::LightcallersCommand);
    g.activate_ability(id, 0);
    g.initiate_raid(RoomId::Vault);
    assert!(g.has(Button::EndRaid));
}

#[test]
fn potentiality_storm() {
    let cost = 0;
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    let id = g.create_and_play(CardName::PotentialityStorm);
    assert!(g.client.cards.get(id).arena_icon().contains("3"));
    g.pass_turn(Side::Riftcaller);
    g.pass_turn(Side::Covenant);
    assert!(g.client.cards.get(id).arena_icon().contains("2"));
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost + 1);
    g.pass_turn(Side::Riftcaller);
    g.pass_turn(Side::Covenant);
    assert!(g.client.cards.get(id).arena_icon().contains("1"));
    g.pass_turn(Side::Riftcaller);
    g.pass_turn(Side::Covenant);
    assert!(matches!(g.client.cards.get(id).position(), Position::DiscardPile(..)));
    assert_eq!(g.client.cards.hand().real_cards().len(), 1);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Crypt);
    g.click(Button::EndRaid);
    g.pass_turn(Side::Riftcaller);
    g.play_card(id, g.user_id(), None);
    assert!(g.client.cards.get(id).arena_icon().contains("3"));
}

#[test]
fn potentiality_storm_cannot_play_without_raids() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    let id = g.create_and_play(CardName::PotentialityStorm);
    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestRitualDestroyAllEnemyPermanents);
    g.pass_turn(Side::Covenant);
    g.pass_turn(Side::Riftcaller);
    assert!(g.play_card_with_result(id, g.user_id(), None).is_err());
}
