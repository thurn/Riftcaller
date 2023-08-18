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

use cards_test::test_cards::{MINION_COST, WEAPON_COST};
use core_ui::icons;
use game_data::card_name::CardName;
use game_data::game_actions::{AccessPhaseAction, EncounterAction, GameAction, PromptAction};
use game_data::primitives::{RoomId, Side};
use insta::assert_snapshot;
use protos::spelldawn::client_action::Action;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    ClientRoomLocation, GainManaAction, InitiateRaidAction, ObjectPositionBrowser,
    ObjectPositionCharacter, ObjectPositionCharacterContainer, ObjectPositionDiscardPile,
    ObjectPositionRaid, ObjectPositionRoom, PlayerName, SpendActionPointAction,
};
use test_utils::client_interface::HasText;
use test_utils::summarize::Summary;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn initiate_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let weapon_id = g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    let (scheme_id, minion_id) = g.setup_raid_target(CardName::TestMinionEndRaid);

    let response = g.initiate_raid(test_constants::ROOM_ID);

    g.click(Buttons::Summon);

    assert_eq!(2, g.me().actions());

    assert!(g.user.this_player.can_take_action());
    assert!(!g.user.other_player.can_take_action());
    assert!(g.opponent.other_player.can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
    assert!(g.user.data.raid_active());
    assert!(g.opponent.data.raid_active());

    assert_eq!(
        g.user.data.object_index_position(Id::CardId(scheme_id)),
        (0, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.opponent.data.object_index_position(Id::CardId(scheme_id)),
        (0, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_index_position(Id::CardId(minion_id)),
        (1, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.opponent.data.object_index_position(Id::CardId(minion_id)),
        (1, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_index_position(Id::Character(PlayerName::User.into())),
        (2, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.opponent.data.object_index_position(Id::Character(PlayerName::Opponent.into())),
        (2, Position::Raid(ObjectPositionRaid {}))
    );

    assert!(g.user.interface.controls().has_text("Test Weapon"));
    assert!(g.user.interface.controls().has_text("Continue"));

    assert_eq!(
        g.legal_actions(Side::Champion),
        vec![
            GameAction::PromptAction(PromptAction::EncounterAction(
                EncounterAction::UseWeaponAbility(
                    test_helpers::server_card_id(weapon_id),
                    test_helpers::server_card_id(minion_id)
                )
            )),
            GameAction::PromptAction(PromptAction::EncounterAction(EncounterAction::NoWeapon))
        ]
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn summon_minion() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let (_, minion_id) = g.setup_raid_target(CardName::TestMinionEndRaid);

    g.initiate_raid(test_constants::ROOM_ID);

    assert!(!g.user.this_player.can_take_action());
    assert!(g.user.other_player.can_take_action());
    assert!(!g.opponent.other_player.can_take_action());
    assert!(g.opponent.this_player.can_take_action());

    assert!(!g.user.cards.get(minion_id).is_face_up());
    assert_eq!(g.user.other_player.mana(), test_constants::STARTING_MANA);

    g.click(Buttons::Summon);

    assert!(g.user.cards.get(minion_id).is_face_up());
    assert_eq!(g.user.other_player.mana(), test_constants::STARTING_MANA - MINION_COST);
    assert!(g.user.this_player.can_take_action());
    assert!(!g.user.other_player.can_take_action());
    assert!(g.opponent.other_player.can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
}

#[test]
fn do_not_summon_minion() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let (_, minion_id) = g.setup_raid_target(CardName::TestMinionEndRaid);

    g.initiate_raid(test_constants::ROOM_ID);
    g.click(Buttons::NoSummon);

    assert!(!g.user.cards.get(minion_id).is_face_up());
    assert_eq!(g.user.other_player.mana(), test_constants::STARTING_MANA);
    assert!(g.user.this_player.can_take_action());
    assert!(!g.user.other_player.can_take_action());
    assert!(g.opponent.other_player.can_take_action());
    assert!(!g.opponent.this_player.can_take_action());
}

#[test]
fn use_weapon() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    let (scheme_id, minion_id) = g.setup_raid_target(CardName::TestMinionEndRaid);

    g.initiate_raid(test_constants::ROOM_ID);
    g.click(Buttons::Summon);

    assert_eq!(g.user.this_player.mana(), 996); // Minion costs 3 to summon
    let response = g.click_on(g.user_id(), "Test Weapon");
    assert_eq!(g.user.this_player.mana(), 995); // Weapon costs 1 to use
    assert_eq!(g.opponent.other_player.mana(), 995); // Weapon costs 1 to use
    assert!(g.user.cards.get(scheme_id).revealed_to_me());
    assert!(g.opponent.cards.get(scheme_id).revealed_to_me());
    assert!(g.user.this_player.can_take_action());
    assert!(g.opponent.other_player.can_take_action());
    assert!(g.user.interface.card_anchor_nodes().has_text("Score!"));
    assert!(g.user.interface.controls().has_text("End Raid"));

    assert_eq!(
        g.user.data.object_index_position(Id::CardId(scheme_id)),
        (0, Position::Browser(ObjectPositionBrowser {}))
    );
    assert_eq!(
        g.user.data.object_position(Id::CardId(minion_id)),
        Position::Room(ObjectPositionRoom {
            room_id: test_constants::CLIENT_ROOM_ID.into(),
            room_location: ClientRoomLocation::Front.into()
        })
    );
    assert_eq!(
        g.user.data.object_position(Id::Character(PlayerName::User.into())),
        Position::CharacterContainer(ObjectPositionCharacterContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn minion_with_shield() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    g.setup_raid_target(CardName::TestMinionEndRaid);
    g.initiate_raid(test_constants::ROOM_ID);
    g.click(Buttons::Summon);
    assert_eq!(g.user.this_player.mana(), test_constants::STARTING_MANA - WEAPON_COST);
    g.click_on(g.user_id(), "Test Weapon");
    assert_eq!(g.user.this_player.mana(), test_constants::STARTING_MANA - WEAPON_COST - 1);
}

#[test]
fn fire_combat_ability() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    let (scheme_id, minion_id) = g.setup_raid_target(CardName::TestMinionEndRaid);

    g.initiate_raid(test_constants::ROOM_ID);
    g.click(Buttons::Summon);

    assert_eq!(g.user.this_player.mana(), 996); // Minion costs 3 to summon
    let response = g.click_on(g.user_id(), "Continue");
    assert_eq!(g.user.this_player.mana(), 996); // Mana is unchanged
    assert_eq!(g.opponent.other_player.mana(), 996);
    assert!(!g.user.cards.get(scheme_id).revealed_to_me()); // Scheme is not revealed

    // Still Champion turn
    assert!(g.user.this_player.can_take_action());
    assert!(g.opponent.other_player.can_take_action());

    assert!(!g.user.data.raid_active()); // No raid active due to End Raid ability
    assert!(!g.opponent.data.raid_active());

    assert_eq!(
        g.user.data.object_position(Id::CardId(minion_id)),
        Position::Room(ObjectPositionRoom {
            room_id: test_constants::CLIENT_ROOM_ID.into(),
            room_location: ClientRoomLocation::Front.into()
        })
    );
    assert_eq!(
        g.user.data.object_position(Id::CardId(scheme_id)),
        Position::Room(ObjectPositionRoom {
            room_id: test_constants::CLIENT_ROOM_ID.into(),
            room_location: ClientRoomLocation::Back.into()
        })
    );
    assert_eq!(
        g.user.data.object_position(Id::Character(PlayerName::User.into())),
        Position::CharacterContainer(ObjectPositionCharacterContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn score_scheme_card() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    let (scheme_id, _) = g.setup_raid_target(CardName::TestMinionEndRaid);
    g.initiate_raid(test_constants::ROOM_ID);
    g.click(Buttons::Summon);

    g.click_on(g.user_id(), "Test Weapon");

    assert_eq!(
        g.legal_actions(Side::Champion),
        vec![
            GameAction::PromptAction(PromptAction::AccessPhaseAction(
                AccessPhaseAction::ScoreCard(test_helpers::server_card_id(scheme_id))
            )),
            GameAction::PromptAction(PromptAction::AccessPhaseAction(AccessPhaseAction::EndRaid))
        ]
    );

    let response = g.click_on(g.user_id(), "Score");

    assert_eq!(g.user.this_player.score(), 15);
    assert_eq!(g.opponent.other_player.score(), 15);
    assert!(g.user.this_player.can_take_action());
    assert!(g.opponent.other_player.can_take_action());
    assert!(g.user.data.raid_active()); // Raid still active
    assert!(g.opponent.data.raid_active());
    assert!(g.user.interface.controls().has_text("End Raid"));

    assert_eq!(
        g.user.data.object_position(Id::CardId(scheme_id)),
        Position::Character(ObjectPositionCharacter { owner: PlayerName::User.into() })
    );
    assert_eq!(
        g.user.data.object_position(Id::Character(PlayerName::User.into())),
        Position::CharacterContainer(ObjectPositionCharacterContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_eq!(
        g.legal_actions(Side::Champion),
        vec![GameAction::PromptAction(PromptAction::AccessPhaseAction(AccessPhaseAction::EndRaid))]
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn complete_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let (scheme_id, _) = g.setup_raid_target(CardName::TestMinionEndRaid);

    // Set up the raid to be the last action of a turn
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    g.perform(Action::SpendActionPoint(SpendActionPointAction {}), g.user_id());
    g.initiate_raid(test_constants::ROOM_ID);
    g.click(Buttons::Summon);

    g.click_on(g.user_id(), "Test Weapon");
    g.click_on(g.user_id(), "Score");
    let response = g.click_on(g.user_id(), "End Raid");

    assert_eq!(g.user.this_player.score(), 15);
    assert_eq!(g.opponent.other_player.score(), 15);
    assert!(g.user.other_player.can_take_action());
    assert!(g.opponent.this_player.can_take_action());
    assert_eq!(g.opponent.interface.main_controls_option(), None);
    assert_eq!(g.user.interface.main_controls_option(), None);
    assert!(!g.user.data.raid_active()); // Raid no longer active
    assert!(!g.opponent.data.raid_active());

    assert_eq!(
        g.user.data.object_position(Id::CardId(scheme_id)),
        Position::Character(ObjectPositionCharacter { owner: PlayerName::User.into() })
    );
    assert_eq!(
        g.user.data.object_position(Id::Character(PlayerName::User.into())),
        Position::CharacterContainer(ObjectPositionCharacterContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn cannot_activate() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(2)
        .opponent(TestSide::new(Side::Overlord).mana(0))
        .build();

    g.create_and_play(CardName::TestScheme3_15);
    g.create_and_play(CardName::TestMinionEndRaid);

    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    let response = g.initiate_raid(test_constants::ROOM_ID);
    assert!(g.user.interface.controls().has_text("Score"));
    assert!(g.user.interface.controls().has_text("End Raid"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn destroy_project() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).current_turn(Side::Overlord).build();
    let project_id = g.create_and_play(CardName::TestProject2Cost);
    g.spend_actions_until_turn_over(Side::Overlord);
    g.initiate_raid(test_constants::ROOM_ID);

    assert!(g.user.interface.controls().has_text("Destroy"));
    assert!(g.user.interface.controls().has_text(format!("2{}", icons::MANA)));

    let response = g.click_on(g.user_id(), "Destroy");
    assert_eq!(
        g.user.data.object_position(Id::CardId(project_id)),
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::Opponent.into() })
    );
    assert_eq!(
        g.opponent.data.object_position(Id::CardId(project_id)),
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::User.into() })
    );
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - 2);
    assert!(g.user.interface.controls().has_text("End Raid"));

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_vault() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(1)
        .opponent(TestSide::new(Side::Overlord).deck_top(CardName::TestScheme3_15))
        .build();

    g.play_with_target_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click(Buttons::Summon);
    let response = g.click_on(g.user_id(), "Test Weapon");
    assert!(g.user.interface.controls().has_text("Score"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_sanctum() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(1)
        .build();

    g.add_to_hand(CardName::TestScheme3_15);
    g.play_with_target_room(CardName::TestMinionEndRaid, RoomId::Sanctum);
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Buttons::Summon);

    let response = g.click_on(g.user_id(), "Test Weapon");
    assert!(g.user.interface.controls().has_text("Score"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_crypts() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(1)
        .opponent(TestSide::new(Side::Overlord).in_discard_face_down(CardName::TestScheme3_15))
        .build();

    g.play_with_target_room(CardName::TestMinionEndRaid, RoomId::Crypts);
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Crypts);
    g.click(Buttons::Summon);

    let response = g.click_on(g.user_id(), "Test Weapon");
    assert!(g.user.interface.controls().has_text("Score"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_vault_twice() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(1)
        .opponent(TestSide::new(Side::Overlord).deck_top(CardName::TestScheme3_15))
        .build();

    g.play_with_target_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click(Buttons::Summon);
    g.click_on(g.user_id(), "Test Weapon");
    g.click_on(g.user_id(), "Score");
    g.click_on(g.user_id(), "End Raid");

    g.initiate_raid(RoomId::Vault);

    // Champion spent mana on playing + using weapon, overlord on summoning
    // minion
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - 4);
    assert_eq!(g.you().mana(), test_constants::STARTING_MANA - 3);

    assert!(g.user.interface.controls().has_text("Test Weapon"));
    g.click_on(g.user_id(), "Test Weapon");

    // Champion spends mana again to use weapon, Overlord mana is unchanged.
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - 5);
    assert_eq!(g.you().mana(), test_constants::STARTING_MANA - 3);

    // Scheme should not longer be on top for second raid
    assert!(g.user.interface.controls().has_text("End Raid"));
    assert!(!g.user.interface.controls().has_text("Score"));
}

#[test]
fn raid_no_defenders() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(1)
        .build();

    g.create_and_play(CardName::TestScheme3_15);
    let response = g.initiate_raid(test_constants::ROOM_ID);
    // Should immediately jump to the Score action
    assert!(g.user.interface.controls().has_text("Score"));
    assert!(g.user.interface.controls().has_text("End Raid"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_vault_no_defenders() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(TestSide::new(Side::Overlord).deck_top(CardName::TestScheme3_15))
        .build();

    g.initiate_raid(RoomId::Vault);
    // Should immediately jump to the Score action
    assert!(g.user.interface.controls().has_text("Score"));
    assert!(g.user.interface.controls().has_text("End Raid"));
}

#[test]
fn raid_no_occupants() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(1)
        .build();

    g.create_and_play(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    let result = g.perform_action(
        Action::InitiateRaid(InitiateRaidAction { room_id: test_constants::CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    test_helpers::assert_error(result);
}

#[test]
fn raid_no_occupants_or_defenders() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();

    let response = g.perform_action(
        Action::InitiateRaid(InitiateRaidAction { room_id: test_constants::CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );

    test_helpers::assert_error(response);
}

#[test]
fn raid_two_defenders() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(2)
        .opponent(TestSide::new(Side::Overlord).deck_top(CardName::TestScheme3_15))
        .build();

    g.play_with_target_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.play_with_target_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click(Buttons::Summon);
    let response = g.click_on(g.user_id(), "Test Weapon");
    g.click(Buttons::Summon);

    assert!(g.user.interface.controls().has_text("Continue"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_two_defenders_full_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(2)
        .opponent(TestSide::new(Side::Overlord).deck_top(CardName::TestScheme3_15))
        .build();

    g.play_with_target_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.play_with_target_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click(Buttons::Summon);
    g.click_on(g.user_id(), "Test Weapon");
    g.click(Buttons::Summon);
    g.click_on(g.user_id(), "Test Weapon");
    let response = g.click_on(g.user_id(), "Score");
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - 5);
    assert_eq!(g.you().mana(), test_constants::STARTING_MANA - 4);
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_deal_damage_game_over() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();

    // Two 'deal 1 damage' defenders are needed because the Champion draws a card
    // for turn
    g.play_with_target_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.play_with_target_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.spend_actions_until_turn_over(Side::Overlord);
    assert!(g.dawn());

    g.initiate_raid(RoomId::Vault);
    g.click(Buttons::Summon);
    g.click_on(g.opponent_id(), "Continue");
    g.click(Buttons::Summon);
    g.click_on(g.opponent_id(), "Continue");

    assert!(g.is_victory_for_player(Side::Overlord));
}

#[test]
fn raid_two_defenders_cannot_afford_second() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(2)
        .opponent(TestSide::new(Side::Overlord).mana(1).deck_top(CardName::TestScheme3_15))
        .build();

    g.play_with_target_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.play_with_target_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click(Buttons::Summon);
    g.click_on(g.user_id(), "Test Weapon");
    let response = g.click_on(g.user_id(), "Score");
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - 4);
    assert_eq!(g.you().mana(), 0);
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_add_defender() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .current_turn(Side::Overlord)
        .actions(2)
        .build();

    g.create_and_play(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TestScheme3_15);
    assert!(g.dawn());

    // Raid 1
    g.initiate_raid(test_constants::ROOM_ID);
    g.click(Buttons::Summon);
    g.click_on(g.user_id(), "Continue");
    assert!(!g.user.data.raid_active());

    g.create_and_play(CardName::TestWeapon3Attack12Boost3Cost);

    // Raid 2, no summon
    g.initiate_raid(test_constants::ROOM_ID);
    g.click_on(g.user_id(), "Test Weapon");
    g.click_on(g.user_id(), "Score");
    g.click_on(g.user_id(), "End Raid");
    assert!(!g.user.data.raid_active());

    // Opponent Turn
    assert!(g.dusk());
    g.create_and_play(CardName::TestMinionDealDamage);
    g.create_and_play(CardName::TestScheme3_15);
    g.perform(Action::GainMana(GainManaAction {}), g.opponent_id());

    // User Turn, Raid 3
    assert!(g.dawn());
    g.initiate_raid(test_constants::ROOM_ID);
    g.click(Buttons::Summon);
    g.click_on(g.user_id(), "Test Weapon");
    let response = g.click_on(g.user_id(), "Test Weapon");
    assert_snapshot!(Summary::summarize(&response));
}
