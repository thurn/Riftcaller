// Copyright © Spelldawn 2021-present

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
pub fn empyreal_chorus() {
    let (cost, gained) = (1, 8);
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .room_occupant(test_constants::ROOM_ID, CardName::TestScheme3_10),
        )
        .build();
    let id = g.create_and_play(CardName::EmpyrealChorus);
    g.activate_ability_with_target(id, 0, test_constants::ROOM_ID);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost + gained);
    assert!(!g.user.data.raid_active());
    assert!(!g.user.cards.room_occupants(test_constants::ROOM_ID)[0].revealed_to_me());
}

#[test]
pub fn starfield_omen() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::StarfieldOmen);
    let id = g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    assert_eq!(g.user.cards.hand().real_cards().len(), 0);
    g.activate_ability(id, 0);
    assert!(g.user.cards.discard_pile().contains_card(CardName::TestSacrificeDrawCardArtifact));
    assert_eq!(g.user.cards.hand().real_cards().len(), 2);
}

#[test]
pub fn visitation() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(1)).build();
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.hand().len(), 1);
    assert!(g.user.cards.discard_pile().contains_card(CardName::Visitation));
}

#[test]
pub fn visitation_pass() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(1)).build();
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    g.click(Button::NoPromptAction);
    assert_eq!(g.user.cards.hand().len(), 0);
    assert!(g.user.cards.evocations_and_allies().contains_card(CardName::Visitation));
}

#[test]
pub fn visitation_multiple_copies() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(1)).build();
    g.create_and_play(CardName::Visitation);
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    g.click(Button::NoPromptAction);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.hand().len(), 1);
    assert!(g.user.cards.discard_pile().contains_card(CardName::Visitation));
    assert!(g.user.cards.evocations_and_allies().contains_card(CardName::Visitation));
}

#[test]
pub fn visitation_prevent_partial() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(5)).build();
    g.create_and_play(CardName::Visitation);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestSpellDeal5Damage);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.hand().len(), 2);
}

#[test]
pub fn visitation_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(5)).build();
    g.create_and_play_upgraded(CardName::Visitation);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestSpellDeal5Damage);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.hand().len(), 5);
}

#[test]
pub fn backup_plan() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();
    let id = g.create_and_play(CardName::BackupPlan);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 0);
    assert!(g.user.data.raid_active());
    g.click(Button::EndRaid);
    assert_eq!(g.me().actions(), 0);
}

#[test]
pub fn backup_plan_upgraded() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .actions(4)
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
        )
        .build();
    let id = g.create_and_play_upgraded(CardName::BackupPlan);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 0);
    assert_eq!(g.me().actions(), 1);
}

#[test]
pub fn backup_plan_cannot_activate_outside_encounter() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(TestSide::new(Side::Overlord))
        .build();
    let id = g.create_and_play(CardName::BackupPlan);
    g.initiate_raid(RoomId::Vault);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
pub fn planar_sanctuary() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord).room_occupant(RoomId::RoomA, CardName::TestScheme3_10),
        )
        .build();
    let id = g.create_and_play(CardName::PlanarSanctuary);
    g.initiate_raid(RoomId::RoomA);
    g.click(Button::Score);
    g.activate_ability(id, 1);
    assert_eq!(g.user.cards.hand().real_cards().len(), 1);
    assert!(g
        .user
        .cards
        .evocations_and_allies()
        .find_card(CardName::PlanarSanctuary)
        .arena_icon()
        .contains('1'));
    g.click(Button::EndRaid);
}

#[test]
pub fn planar_sanctuary_activate_after_curse() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let id = g.create_and_play(CardName::PlanarSanctuary);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    assert!(g
        .user
        .cards
        .evocations_and_allies()
        .find_card(CardName::PlanarSanctuary)
        .arena_icon()
        .contains('2'));
    g.create_and_play(CardName::TestSpellGiveCurse);
    assert!(g.me().can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
    assert_eq!(g.user.cards.hand().curse_count(), 1);
    assert_eq!(g.user.cards.hand().real_cards().len(), 0);
    g.activate_ability(id, 1);
    assert_eq!(g.user.cards.hand().curse_count(), 0);
    assert_eq!(g.user.cards.hand().real_cards().len(), 1);
    assert!(g.me().can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
    assert!(g
        .user
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
pub fn planar_sanctuary_activate_after_damage() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(5)).build();
    let id = g.create_and_play(CardName::PlanarSanctuary);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    assert_eq!(g.user.cards.hand().real_cards().len(), 4);
    assert!(g.me().can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
    g.activate_ability(id, 1);
    assert_eq!(g.user.cards.hand().real_cards().len(), 5);
    assert!(g.me().can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
    g.click(Button::ClosePriorityPrompt);
    assert!(!g.me().can_take_action());
    assert!(g.opponent.this_player.can_take_action());
}

#[test]
pub fn knowledge_of_the_beyond() {
    let (cost, reduction) = (0, 1);
    let mut g = TestGame::new(
        TestSide::new(Side::Champion)
            .deck_top(CardName::TestSacrificeDrawCardArtifact)
            .deck_top(CardName::TestEvocation),
    )
    .build();

    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.activate_ability(id, 1);
    let artifact_id = g.user.cards.hand().find_card_id(CardName::TestSacrificeDrawCardArtifact);
    g.play_card(artifact_id, g.user_id(), None);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA - cost - (test_constants::ARTIFACT_COST - reduction)
    );
    assert!(g.user.cards.discard_pile().contains_card(CardName::KnowledgeOfTheBeyond));
    assert!(g.user.cards.discard_pile().contains_card(CardName::TestEvocation));
    assert!(g.user.cards.artifacts().contains_card(CardName::TestSacrificeDrawCardArtifact));
}

#[test]
pub fn knowledge_of_the_beyond_activate_for_weapon_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).deck_top(CardName::TestMortalWeapon))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();

    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 1);
    let artifact_id = g.user.cards.hand().find_card_id(CardName::TestMortalWeapon);
    g.play_card(artifact_id, g.user_id(), None);
    g.click_card_name(CardName::TestMortalWeapon);
    assert!(g.user.data.raid_active());
    g.click(Button::EndRaid);
}

#[test]
pub fn knowledge_of_the_beyond_activate_during_access() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).deck_top(CardName::TestMortalWeapon))
        .opponent(TestSide::new(Side::Overlord).deck_top(CardName::TestScheme3_10))
        .build();

    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 1);
    let artifact_id = g.user.cards.hand().find_card_id(CardName::TestMortalWeapon);
    g.play_card(artifact_id, g.user_id(), None);
    assert!(g.user.cards.artifacts().contains_card(CardName::TestMortalWeapon));
    g.click(Button::Score);
}

#[test]
pub fn knowledge_of_the_beyond_no_hits() {
    let mut g = TestGame::new(
        TestSide::new(Side::Champion)
            .deck_top(CardName::TestSpell)
            .deck_top(CardName::TestSpell)
            .deck_top(CardName::TestSpell)
            .hand_size(5),
    )
    .build();

    // You can still activate it with no hits to e.g. put the cards into your
    // discard pile
    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.activate_ability(id, 1);
    assert_eq!(g.user.cards.hand().len(), 0);
    g.click(Button::SkipPlayingCard);
    assert_eq!(g.user.cards.discard_pile().len(), 4);
}

#[test]
pub fn knowledge_of_the_beyond_activate_planar_sanctuary() {
    let mut g = TestGame::new(
        TestSide::new(Side::Champion).deck_top(CardName::TestMortalWeapon).hand_size(5),
    )
    .build();
    let knowledge_of_the_beyond = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.create_and_play(CardName::PlanarSanctuary);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    g.activate_ability(knowledge_of_the_beyond, 1);
    let artifact_id = g.user.cards.hand().find_card_id(CardName::TestMortalWeapon);
    g.play_card(artifact_id, g.user_id(), None);
    assert!(g.user.cards.artifacts().contains_card(CardName::TestMortalWeapon));
    assert!(g.user.cards.artifacts().find_card(CardName::TestMortalWeapon).is_face_up());
    g.click(Button::ClosePriorityPrompt);
}

#[test]
pub fn knowledge_of_the_beyond_activate_planar_sanctuary_first() {
    let mut g = TestGame::new(
        TestSide::new(Side::Champion).deck_top(CardName::TestMortalWeapon).hand_size(5),
    )
    .build();
    let knowledge_of_the_beyond = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    let planar_sanctuary = g.create_and_play(CardName::PlanarSanctuary);
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestScheme1_10);
    g.progress_room(test_constants::ROOM_ID);
    g.create_and_play(CardName::TestSpellDeal1Damage);
    g.activate_ability(planar_sanctuary, 1);
    g.activate_ability(knowledge_of_the_beyond, 1);
    let artifact_id = g.user.cards.hand().find_card_id(CardName::TestMortalWeapon);
    g.play_card(artifact_id, g.user_id(), None);
    assert!(g.user.cards.artifacts().contains_card(CardName::TestMortalWeapon));
    assert!(g.user.cards.artifacts().find_card(CardName::TestMortalWeapon).is_face_up());
    g.click(Button::ClosePriorityPrompt);
}

#[test]
pub fn knowledge_of_the_beyond_card_target_with_foebane() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).deck_top(CardName::Foebane))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Vault, CardName::TestMortalMinion),
        )
        .build();

    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 1);
    let foebane = g.user.cards.hand().find_card_id(CardName::Foebane);
    g.play_card(foebane, g.user_id(), test_helpers::target_room(RoomId::Vault));
    g.click(Button::ChooseOnPlay);

    // This is a bit weird, but basically Foebane triggers during the approach step,
    // which is what allows it to by pass "on encounter" abilities, so you can't put
    // it into play and then use it immediately during an encounter. We could prompt
    // *again* to use it on encounter but that would be very annoying.
    assert!(!g.has(Button::Evade));
}

#[test]
pub fn knowledge_of_the_beyond_shield_of_the_flames_evade() {
    let mut g = TestGame::new(
        TestSide::new(Side::Champion)
            .deck_top(CardName::ShieldOfTheFlames)
            .deck_top(CardName::TestSacrificeDrawCardArtifact),
    )
    .opponent(
        TestSide::new(Side::Overlord).face_up_defender(RoomId::Vault, CardName::TestInfernalMinion),
    )
    .build();

    let id = g.create_and_play(CardName::KnowledgeOfTheBeyond);
    g.initiate_raid(RoomId::Vault);
    g.activate_ability(id, 1);
    let shield = g.user.cards.hand().find_card_id(CardName::ShieldOfTheFlames);
    g.play_card(shield, g.user_id(), None);

    // Unlike with Foebane you *can* evade a minion with Shield of the Flames
    // because this ability happens *during* an encounter and does not bypass
    // encounter triggers.
    g.activate_ability(shield, 0);
    assert!(g.user.cards.discard_pile().contains_card(CardName::ShieldOfTheFlames));
    assert!(g.user.cards.discard_pile().contains_card(CardName::KnowledgeOfTheBeyond));
    assert!(g.user.cards.discard_pile().contains_card(CardName::TestSacrificeDrawCardArtifact));
    assert!(g.user.data.raid_active());
    g.click(Button::EndRaid);
}
