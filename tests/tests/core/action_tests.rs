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

use game_data::card_name::CardName;
use game_data::game_actions;
use game_data::game_actions::GameAction;
use game_data::primitives::{RoomId, Side};
use insta::assert_snapshot;
use protos::spelldawn::client_action::Action;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    card_target, CardTarget, ClientRoomLocation, DrawCardAction, GainManaAction, GameMessageType,
    LevelUpRoomAction, ObjectPositionDiscardPile, PlayCardAction, PlayerName,
};
use test_utils::summarize::Summary;
use test_utils::test_game::{TestGame, TestRaid, TestSide};
use test_utils::test_session_builder::TestSessionBuilder;
use test_utils::*;

#[test]
fn connect() {
    let mut g = TestSessionBuilder::new()
        .do_not_connect(true)
        .game(TestGame::new(TestSide::new(Side::Overlord)))
        .build();
    let response = g.connect(g.user_id());
    assert_snapshot!(Summary::run(&response));
}

#[test]
fn connect_to_ongoing() {
    let mut g = TestGame::new(
        TestSide::new(Side::Overlord).deck_top(CardName::TestMinionDealDamageEndRaid),
    )
    .build();
    let r1 = g.connect(g.user_id());
    test_helpers::assert_ok(&r1);
    let r2 = g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id());
    test_helpers::assert_identical(
        vec![CardName::TestMinionDealDamageEndRaid],
        g.user.cards.hand(PlayerName::User),
    );
    test_helpers::assert_ok(&r2);
    let r3 = g.connect(g.opponent_id());

    assert_snapshot!(Summary::run(&r3));
}

#[test]
fn draw_card() {
    let mut g = TestGame::new(
        TestSide::new(Side::Overlord).deck_top(CardName::TestMinionDealDamageEndRaid),
    )
    .build();
    let response = g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id());
    assert_snapshot!(Summary::run(&response));

    test_helpers::assert_identical(
        vec![CardName::TestMinionDealDamageEndRaid],
        g.user.cards.hand(PlayerName::User),
    );
    assert_eq!(vec![test_constants::HIDDEN_CARD], g.opponent.cards.hand(PlayerName::Opponent));
    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
}

#[test]
fn cannot_draw_card_on_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    test_helpers::assert_error(
        g.perform_action(Action::DrawCard(DrawCardAction {}), g.opponent_id()),
    );
}

#[test]
fn cannot_draw_when_out_of_action_points() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(0).build();
    test_helpers::assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id()));
}

#[test]
fn cannot_draw_during_raid() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord)).actions(0).raid(TestRaid::new()).build();
    test_helpers::assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id()));
}

#[test]
fn maximum_hand_size() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).deck_top(CardName::TestMinionEndRaid)).build();
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.pass_turn(Side::Overlord);
    g.pass_turn(Side::Champion);
    assert_eq!(4, g.user.cards.hand(PlayerName::User).len());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.pass_turn(Side::Overlord);
    g.pass_turn(Side::Champion);
    assert_eq!(8, g.user.cards.hand(PlayerName::User).len());
    g.pass_turn(Side::Overlord);
    assert_eq!(vec!["Test Minion End Raid"], g.user.cards.discard_pile(PlayerName::User));
}

#[test]
fn play_card() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).mana(5)).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    let response = g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    );
    assert_snapshot!(Summary::run(&response));

    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(9, g.me().mana());
    assert_eq!(9, g.opponent.other_player.mana());
    test_helpers::assert_identical(
        vec![CardName::ArcaneRecovery],
        g.user.cards.discard_pile(PlayerName::User),
    );
    test_helpers::assert_identical(
        vec![CardName::ArcaneRecovery],
        g.opponent.cards.discard_pile(PlayerName::Opponent),
    );
}

#[test]
fn play_hidden_card() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(0)).build();
    let card_id = g.add_to_hand(CardName::GoldMine);
    let response = g.perform_action(
        Action::PlayCard(PlayCardAction {
            card_id: Some(card_id),
            target: Some(CardTarget {
                card_target: Some(card_target::CardTarget::RoomId(
                    test_constants::CLIENT_ROOM_ID.into(),
                )),
            }),
        }),
        g.user_id(),
    );
    assert_snapshot!(Summary::run(&response));

    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(0, g.me().mana());
    assert_eq!(0, g.opponent.other_player.mana());
    test_helpers::assert_identical(
        vec![CardName::GoldMine],
        g.user.cards.room_cards(test_constants::ROOM_ID, ClientRoomLocation::Back),
    );
    assert_eq!(
        vec![test_constants::HIDDEN_CARD],
        g.opponent.cards.room_cards(test_constants::ROOM_ID, ClientRoomLocation::Back)
    );
}

#[test]
fn cannot_play_card_on_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    test_helpers::assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn cannot_play_card_when_out_of_action_points() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).actions(0).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    test_helpers::assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn cannot_play_card_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).raid(TestRaid::new()).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    test_helpers::assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn gain_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(5)).build();
    let response = g.perform_action(Action::GainMana(GainManaAction {}), g.user_id());

    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(6, g.me().mana());
    assert_eq!(6, g.opponent.other_player.mana());

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn cannot_gain_mana_on_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    test_helpers::assert_error(
        g.perform_action(Action::GainMana(GainManaAction {}), g.opponent_id()),
    );
}

#[test]
fn cannot_gain_mana_when_out_of_action_points() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(0).build();
    test_helpers::assert_error(g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()));
}

#[test]
fn cannot_gain_mana_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).raid(TestRaid::new()).build();
    test_helpers::assert_error(g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()));
}

#[test]
fn level_up_room() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(10)).build();
    g.create_and_play(CardName::TestScheme3_15);
    let response = g.perform_action(
        Action::LevelUpRoom(LevelUpRoomAction { room_id: test_constants::CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );

    assert_snapshot!(Summary::run(&response));
    assert_eq!(g.user.this_player.mana(), 9);
    assert_eq!(g.opponent.other_player.mana(), 9);
}

#[test]
fn minion_limit() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(6).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TestInfernalMinion);
    g.create_and_play(CardName::TestAbyssalMinion);
    g.create_and_play(CardName::TestMortalMinion);
    assert_eq!(
        g.user.cards.room_cards(test_constants::ROOM_ID, ClientRoomLocation::Front).len(),
        4
    );
    g.create_and_play(CardName::TestMinionDealDamage);
    assert_eq!(
        g.user.cards.room_cards(test_constants::ROOM_ID, ClientRoomLocation::Front).len(),
        4
    );
    assert_eq!(g.user.cards.discard_pile(PlayerName::User), vec!["Test Minion End Raid"]);
}

#[test]
fn score_overlord_card() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(10)).actions(5).build();
    let scheme_id = g.create_and_play(CardName::TestScheme3_15);
    let level_up =
        Action::LevelUpRoom(LevelUpRoomAction { room_id: test_constants::CLIENT_ROOM_ID.into() });
    g.perform(level_up.clone(), g.user_id());
    g.perform(level_up.clone(), g.user_id());
    let response = g.perform_action(level_up, g.user_id());

    assert_snapshot!(Summary::run(&response));
    assert!(g.opponent.cards.get(scheme_id).revealed_to_me());
    assert_eq!(g.user.this_player.mana(), 7);
    assert_eq!(g.opponent.other_player.mana(), 7);
    assert_eq!(g.user.this_player.score(), 15);
    assert_eq!(g.opponent.other_player.score(), 15);
}

#[test]
fn overlord_win_game() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(10).score(90)).actions(5).build();
    g.create_and_play(CardName::TestScheme3_15);
    let level_up =
        Action::LevelUpRoom(LevelUpRoomAction { room_id: test_constants::CLIENT_ROOM_ID.into() });
    g.perform(level_up.clone(), g.user_id());
    g.perform(level_up.clone(), g.user_id());
    let response = g.perform_action(level_up, g.user_id());

    assert_snapshot!(Summary::run(&response));
    assert_eq!(g.user.data.last_message(), GameMessageType::Victory);
    assert_eq!(g.opponent.data.last_message(), GameMessageType::Defeat);
}

#[test]
fn switch_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(5)).actions(3).build();
    g.perform(Action::GainMana(GainManaAction {}), g.user_id());
    g.perform(Action::GainMana(GainManaAction {}), g.user_id());
    g.perform(Action::GainMana(GainManaAction {}), g.user_id());
    g.click(Button::EndTurn);

    assert_eq!(8, g.me().mana());
    assert_eq!(8, g.opponent.other_player.mana());
    assert_eq!(0, g.me().actions());
    assert_eq!(0, g.opponent.other_player.actions());
    assert_eq!(4, g.user.other_player.actions());
    assert_eq!(4, g.opponent.this_player.actions());
    assert!(!g.user.this_player.can_take_action());
    assert!(g.user.other_player.can_take_action());
    assert!(g.opponent.this_player.can_take_action());
    assert!(!g.opponent.other_player.can_take_action());
}

#[test]
fn activate_ability() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).actions(3).build();
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    let ability_card_id = g
        .user
        .cards
        .cards_in_hand(PlayerName::User)
        .find(|c| c.id().ability_id.is_some())
        .expect("ability card")
        .id();

    assert_eq!(test_constants::STARTING_MANA - test_constants::ARTIFACT_COST, g.me().mana());
    assert_eq!(2, g.me().actions());

    let response = g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(ability_card_id), target: None }),
        g.user_id(),
    );

    assert_snapshot!(Summary::run(&response));
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::ARTIFACT_COST + test_constants::MANA_TAKEN,
        g.me().mana()
    );
    assert_eq!(1, g.me().actions());
}

#[test]
fn activate_ability_take_all_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).actions(3).build();
    let id = g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    let ability_card_id = g
        .user
        .cards
        .cards_in_hand(PlayerName::User)
        .find(|c| c.id().ability_id.is_some())
        .expect("ability card")
        .id();

    let mut taken = 0;
    while taken < test_constants::MANA_STORED {
        g.perform(
            Action::PlayCard(PlayCardAction { card_id: Some(ability_card_id), target: None }),
            g.user_id(),
        );
        taken += test_constants::MANA_TAKEN;

        g.pass_turn(Side::Champion);
        assert!(g.dusk());
        g.pass_turn(Side::Overlord);
        assert!(g.dawn());
    }

    assert_eq!(
        test_constants::STARTING_MANA - test_constants::ARTIFACT_COST + test_constants::MANA_STORED,
        g.user.this_player.mana()
    );
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::ARTIFACT_COST + test_constants::MANA_STORED,
        g.opponent.other_player.mana()
    );
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::User.into() }),
        g.user.cards.get(id).position()
    );
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::Opponent.into() }),
        g.opponent.cards.get(id).position()
    );
}

#[test]
fn unveil_at_end_of_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestTriggeredAbilityTakeManaAtDusk);
    assert_eq!(test_constants::STARTING_MANA, g.user.this_player.mana());
    g.pass_turn(Side::Overlord);
    assert!(g.dawn());
    g.spend_all_action_points(Side::Champion);
    g.opponent_click(Button::EndTurn);
    assert_eq!(test_constants::STARTING_MANA, g.user.this_player.mana());
    g.unveil_card(id);
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::UNVEIL_COST,
        g.user.this_player.mana()
    );
    g.click(Button::StartTurn);
    assert!(g.dusk());
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::UNVEIL_COST + test_constants::MANA_TAKEN,
        g.user.this_player.mana()
    );
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::UNVEIL_COST + test_constants::MANA_TAKEN,
        g.opponent.other_player.mana()
    );
}

#[test]
fn unveil_during_minion_summon_decision() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    let id = g.create_and_play(CardName::TestSummonboundProject);
    g.pass_turn(Side::Overlord);

    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.user.this_player.mana());
    g.unveil_card(id);
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::UNVEIL_COST,
        g.user.this_player.mana()
    );
}

#[test]
fn cannot_unveil_duskbound_during_minion_summon_decision() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    let id = g.create_and_play(CardName::TestDuskboundProject);
    g.pass_turn(Side::Overlord);
    g.initiate_raid(test_constants::ROOM_ID);
    assert!(g.unveil_card_with_result(id).is_err());
}

#[test]
fn unveil_during_room_approach() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestRoomboundProject);
    g.pass_turn(Side::Overlord);
    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.user.this_player.mana());
    g.unveil_card(id);
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::UNVEIL_COST,
        g.user.this_player.mana()
    );
    g.click(Button::ProceedToAccess);
    g.opponent_click(Button::EndRaid);
}

#[test]
fn cannot_unveil_summonbound_during_room_approach() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestSummonboundProject);
    g.pass_turn(Side::Overlord);
    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.user.this_player.mana());
    assert!(g.unveil_card_with_result(id).is_err());
}

#[test]
fn cannot_unveil_duskbound_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    let id = g.create_and_play(CardName::TestDuskboundProject);
    g.pass_turn(Side::Overlord);

    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.user.this_player.mana());
    assert!(g.unveil_card_with_result(id).is_err());
}

#[test]
fn cannot_unveil_nightbound_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    let id = g.create_and_play(CardName::TestNightboundProject);
    g.pass_turn(Side::Overlord);

    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.user.this_player.mana());
    assert!(g.unveil_card_with_result(id).is_err());
}

#[test]
fn cannot_unveil_trap_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    let id = g.create_and_play(CardName::TestTrapProject);
    g.pass_turn(Side::Overlord);

    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.user.this_player.mana());
    assert!(g.unveil_card_with_result(id).is_err());
}

#[test]
fn triggered_ability_cannot_unveil() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(0)).actions(1).build();
    g.create_and_play(CardName::TestTriggeredAbilityTakeManaAtDusk);
    g.pass_turn(Side::Overlord);
    assert!(g.dawn());
    assert_eq!(0, g.user.this_player.mana());
    g.pass_turn(Side::Champion);
    assert!(g.dusk());
    assert_eq!(0, g.user.this_player.mana());
    assert_eq!(0, g.opponent.other_player.mana());
}

#[test]
fn cannot_unveil_duskbound_immediately() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestDuskboundProject);
    assert!(g.unveil_card_with_result(id).is_err());
}

#[test]
fn cannot_unveil_trap_immediately() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestTrapProject);
    assert!(g.unveil_card_with_result(id).is_err());
}

#[test]
fn unveil_nightbound_immediately() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestNightboundProject);
    assert!(g.unveil_card_with_result(id).is_ok());
}

#[test]
fn unveil_duskbound_at_end_of_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestDuskboundProject);
    g.pass_turn(Side::Overlord);
    g.to_end_step(Side::Champion);
    assert!(g.dawn()); // Game should pause on end step
    assert!(g.side_has(Button::StartTurn, Side::Overlord));
    assert!(g.unveil_card_with_result(id).is_ok());
}

#[test]
fn cannot_unveil_nightbound_at_end_of_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestNightboundProject);
    g.pass_turn(Side::Overlord);
    g.to_end_step(Side::Champion);
    assert!(g.dusk()); // Game should *not* pause on end step
}

#[test]
fn cannot_unveil_trap_at_end_of_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestTrapProject);
    g.pass_turn(Side::Overlord);
    g.to_end_step(Side::Champion);
    assert!(g.dusk()); // Game should *not* pause on end step
}

#[test]
fn triggered_ability_take_all_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(1).build();
    let id = g.create_and_play(CardName::TestTriggeredAbilityTakeManaAtDusk);
    g.pass_turn(Side::Overlord);
    g.to_end_step(Side::Champion);
    g.unveil_card(id);
    g.click(Button::StartTurn);

    let mut taken = 0;
    while taken < test_constants::MANA_STORED {
        assert!(g.dusk());
        taken += test_constants::MANA_TAKEN;
        g.pass_turn(Side::Overlord);
        assert!(g.dawn());
        g.pass_turn(Side::Champion);
    }

    assert_eq!(
        test_constants::STARTING_MANA - test_constants::UNVEIL_COST + test_constants::MANA_STORED,
        g.user.this_player.mana()
    );
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::UNVEIL_COST + test_constants::MANA_STORED,
        g.opponent.other_player.mana()
    );
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::User.into() }),
        g.user.cards.get(id).position()
    );
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::Opponent.into() }),
        g.opponent.cards.get(id).position()
    );
}

#[test]
fn use_artifact_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).current_turn(Side::Overlord).build();
    g.create_and_play(CardName::TestScheme3_15);
    g.create_and_play(CardName::TestMinionEndRaid);
    g.pass_turn(Side::Overlord);
    let id = g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::Summon);
    assert_eq!(g.user.cards.real_cards_in_hand_count(), 0);
    assert_eq!(g.user.cards.ability_cards_in_hand_count(), 1);
    g.activate_ability(id, 0);
    assert_eq!(g.user.cards.real_cards_in_hand_count(), 1);
    assert_eq!(g.user.cards.ability_cards_in_hand_count(), 0);
}

#[test]
fn cannot_use_action_artifact_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).current_turn(Side::Overlord).build();
    g.create_and_play(CardName::TestScheme3_15);
    g.create_and_play(CardName::TestMinionEndRaid);
    g.pass_turn(Side::Overlord);
    let id = g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::Summon);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn legal_actions() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    assert!(g.legal_actions_result(Side::Champion).is_err());
    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Overlord),
        vec![GameAction::GainMana, GameAction::DrawCard],
    );

    let spell_id = test_helpers::server_card_id(g.add_to_hand(CardName::TestOverlordSpell));

    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Overlord),
        vec![
            GameAction::GainMana,
            GameAction::DrawCard,
            GameAction::PlayCard(spell_id, game_actions::CardTarget::None),
        ],
    );

    let minion_id = test_helpers::server_card_id(g.add_to_hand(CardName::TestMinionEndRaid));

    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Overlord),
        vec![
            GameAction::GainMana,
            GameAction::DrawCard,
            GameAction::PlayCard(spell_id, game_actions::CardTarget::None),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::Sanctum)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::Vault)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::Crypts)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::RoomA)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::RoomB)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::RoomC)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::RoomD)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::RoomE)),
        ],
    );
}

#[test]
fn legal_actions_level_up_room() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestScheme3_15);
    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Overlord),
        vec![GameAction::GainMana, GameAction::DrawCard, GameAction::LevelUpRoom(RoomId::RoomA)],
    );
}

#[test]
fn champion_legal_actions() {
    let g = TestGame::new(TestSide::new(Side::Champion)).build();
    assert!(g.legal_actions_result(Side::Overlord).is_err());
    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Champion),
        vec![
            GameAction::GainMana,
            GameAction::DrawCard,
            GameAction::InitiateRaid(RoomId::Sanctum),
            GameAction::InitiateRaid(RoomId::Vault),
            GameAction::InitiateRaid(RoomId::Crypts),
        ],
    );
}
