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
use game_data::game_actions;
use game_data::game_actions::GameAction;
use insta::assert_snapshot;
use protos::riftcaller::client_action::Action;
use protos::riftcaller::object_position::Position;
use protos::riftcaller::{
    card_target, CardTarget, DrawCardAction, GainManaAction, GameMessageType,
    ObjectPositionDiscardPile, PlayCardAction, PlayerName, ProgressRoomAction,
};
use test_utils::summarize::Summary;
use test_utils::test_game::{TestGame, TestRaid, TestSide};
use test_utils::test_session_builder::TestSessionBuilder;
use test_utils::*;

#[test]
fn connect() {
    let mut g = TestSessionBuilder::new()
        .do_not_connect(true)
        .game(TestGame::new(TestSide::new(Side::Covenant)))
        .build();
    let response = g.connect(g.user_id());
    assert_snapshot!(Summary::run(&response));
}

#[test]
fn connect_to_ongoing() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant).deck_top(CardName::TestMinionDealDamageEndRaid),
    )
    .build();
    let r1 = g.connect(g.user_id());
    test_helpers::assert_ok(&r1);
    let r2 = g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id());
    test_helpers::assert_cards_match(
        g.client.cards.hand(),
        vec![CardName::TestMinionDealDamageEndRaid],
    );
    test_helpers::assert_ok(&r2);
    let r3 = g.connect(g.opponent_id());

    assert_snapshot!(Summary::run(&r3));
}

#[test]
fn draw_card() {
    let mut g = TestGame::new(
        TestSide::new(Side::Covenant).deck_top(CardName::TestMinionDealDamageEndRaid),
    )
    .build();
    let response = g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id());
    assert_snapshot!(Summary::run(&response));

    test_helpers::assert_cards_match(
        g.client.cards.hand(),
        vec![CardName::TestMinionDealDamageEndRaid],
    );
    assert_eq!(vec![test_constants::HIDDEN_CARD], g.opponent.cards.opponent_hand().names());
    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
}

#[test]
fn cannot_draw_card_on_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    test_helpers::assert_error(
        g.perform_action(Action::DrawCard(DrawCardAction {}), g.opponent_id()),
    );
}

#[test]
fn cannot_draw_when_out_of_action_points() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(0).build();
    test_helpers::assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id()));
}

#[test]
fn cannot_draw_during_raid() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant)).actions(0).raid(TestRaid::new()).build();
    test_helpers::assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id()));
}

#[test]
fn play_card() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).mana(5)).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    let response = g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    );
    assert_snapshot!(Summary::run(&response));

    assert_eq!(3, g.me().actions());
    assert_eq!(3, g.opponent.other_player.actions());
    assert_eq!(9, g.me().mana());
    assert_eq!(9, g.opponent.other_player.mana());
    test_helpers::assert_cards_match(g.client.cards.discard_pile(), vec![CardName::ArcaneRecovery]);
    test_helpers::assert_cards_match(
        g.opponent.cards.opponent_discard_pile(),
        vec![CardName::ArcaneRecovery],
    );
}

#[test]
fn play_hidden_card() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).mana(0)).build();
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
    test_helpers::assert_cards_match(
        g.client.cards.room_occupants(test_constants::ROOM_ID),
        vec![CardName::GoldMine],
    );
    assert_eq!(
        vec![test_constants::HIDDEN_CARD],
        g.opponent.cards.room_occupants(test_constants::ROOM_ID).names()
    );
}

#[test]
fn cannot_play_card_on_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    test_helpers::assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn cannot_play_card_when_out_of_action_points() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).actions(0).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    test_helpers::assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn cannot_play_card_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).raid(TestRaid::new()).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    test_helpers::assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn gain_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).mana(5)).build();
    let response = g.perform_action(Action::GainMana(GainManaAction {}), g.user_id());

    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(6, g.me().mana());
    assert_eq!(6, g.opponent.other_player.mana());

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn cannot_gain_mana_on_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    test_helpers::assert_error(
        g.perform_action(Action::GainMana(GainManaAction {}), g.opponent_id()),
    );
}

#[test]
fn cannot_gain_mana_when_out_of_action_points() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(0).build();
    test_helpers::assert_error(g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()));
}

#[test]
fn cannot_gain_mana_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).raid(TestRaid::new()).build();
    test_helpers::assert_error(g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()));
}

#[test]
fn progress_room() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).mana(10)).build();
    g.create_and_play(CardName::TestScheme3_10);
    let response = g.perform_action(
        Action::ProgressRoom(ProgressRoomAction { room_id: test_constants::CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );

    assert_snapshot!(Summary::run(&response));
    assert_eq!(g.client.this_player.mana(), 9);
    assert_eq!(g.opponent.other_player.mana(), 9);
}

#[test]
fn minion_limit() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(6).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TestInfernalMinion);
    g.create_and_play(CardName::TestAstralMinion);
    g.create_and_play(CardName::TestMortalMinion);
    assert_eq!(g.client.cards.room_defenders(test_constants::ROOM_ID).len(), 4);
    g.create_and_play(CardName::TestMinionDealDamage);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.room_defenders(test_constants::ROOM_ID).len(), 4);
    assert_eq!(g.client.cards.discard_pile().len(), 1);
}

#[test]
fn minion_limit_cannot_take_other_actions() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(6).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TestInfernalMinion);
    g.create_and_play(CardName::TestAstralMinion);
    g.create_and_play(CardName::TestMortalMinion);
    g.create_and_play(CardName::TestMinionDealDamage);
    assert!(g.draw_card_with_result().is_err());
}

#[test]
fn minion_limit_cancel_playing() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(6).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TestInfernalMinion);
    g.create_and_play(CardName::TestAstralMinion);
    g.create_and_play(CardName::TestMortalMinion);
    g.create_and_play(CardName::TestMinionDealDamage);
    g.click(Button::CancelPlayingCard);
    assert_eq!(g.client.cards.room_defenders(test_constants::ROOM_ID).len(), 4);
    assert_eq!(g.client.cards.hand().real_cards().len(), 1);
    assert!(g.draw_card_with_result().is_ok());
}

#[test]
fn weapon_limit() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).actions(6).build();
    g.create_and_play(CardName::TestWeapon2Attack);
    g.create_and_play(CardName::TestWeapon2Attack);
    g.create_and_play(CardName::TestWeapon2Attack);
    g.create_and_play(CardName::TestWeapon2Attack);
    g.create_and_play(CardName::TestWeapon2Attack);
    assert_eq!(g.client.cards.discard_pile().len(), 0);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.discard_pile().len(), 1);
}

#[test]
fn evocation_limit() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).actions(6).build();
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    assert_eq!(g.client.cards.discard_pile().len(), 0);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.discard_pile().len(), 1);
}

#[test]
fn sacrifice_existing_project() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::TestScheme3_10);
    g.create_and_play(CardName::TestProject2Cost3Raze);
    assert_eq!(g.client.cards.discard_pile().len(), 0);
    g.click(Button::Sacrifice);
    assert_eq!(g.client.cards.discard_pile().names(), vec!["Test Scheme 3_10"]);
}

#[test]
fn score_covenant_card() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).mana(10)).actions(5).build();
    let scheme_id = g.create_and_play(CardName::TestScheme3_10);
    let progress =
        Action::ProgressRoom(ProgressRoomAction { room_id: test_constants::CLIENT_ROOM_ID.into() });
    g.perform(progress.clone(), g.user_id());
    g.perform(progress.clone(), g.user_id());
    let response = g.perform_action(progress, g.user_id());

    assert_snapshot!(Summary::run(&response));
    assert!(g.opponent.cards.get(scheme_id).revealed_to_me());
    assert_eq!(g.client.this_player.mana(), 7);
    assert_eq!(g.opponent.other_player.mana(), 7);
    assert_eq!(g.client.this_player.score(), 10);
    assert_eq!(g.opponent.other_player.score(), 10);
}

#[test]
fn covenant_win_game() {
    let mut g =
        TestGame::new(TestSide::new(Side::Covenant).mana(10).bonus_points(90)).actions(5).build();
    g.create_and_play(CardName::TestScheme3_10);
    let progress =
        Action::ProgressRoom(ProgressRoomAction { room_id: test_constants::CLIENT_ROOM_ID.into() });
    g.perform(progress.clone(), g.user_id());
    g.perform(progress.clone(), g.user_id());
    let response = g.perform_action(progress, g.user_id());

    assert_snapshot!(Summary::run(&response));
    assert_eq!(g.client.data.last_message(), GameMessageType::Victory);
    assert_eq!(g.opponent.data.last_message(), GameMessageType::Defeat);
}

#[test]
fn switch_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).mana(5)).actions(3).build();
    g.perform(Action::GainMana(GainManaAction {}), g.user_id());
    g.perform(Action::GainMana(GainManaAction {}), g.user_id());
    g.perform(Action::GainMana(GainManaAction {}), g.user_id());
    g.click(Button::EndTurn);

    assert_eq!(8, g.me().mana());
    assert_eq!(8, g.opponent.other_player.mana());
    assert_eq!(0, g.me().actions());
    assert_eq!(0, g.opponent.other_player.actions());
    assert_eq!(4, g.client.other_player.actions());
    assert_eq!(4, g.opponent.this_player.actions());
    assert!(!g.client.this_player.can_take_action());
    assert!(g.client.other_player.can_take_action());
    assert!(g.opponent.this_player.can_take_action());
    assert!(!g.opponent.other_player.can_take_action());
}

#[test]
fn activate_ability() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).actions(3).build();
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    let ability_card_id = g
        .client
        .cards
        .hand()
        .into_iter()
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
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).actions(3).build();
    let id = g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    let ability_card_id = g
        .client
        .cards
        .hand()
        .into_iter()
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

        g.pass_turn(Side::Riftcaller);
        assert!(g.dusk());
        g.pass_turn(Side::Covenant);
        assert!(g.dawn());
    }

    assert_eq!(
        test_constants::STARTING_MANA - test_constants::ARTIFACT_COST + test_constants::MANA_STORED,
        g.client.this_player.mana()
    );
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::ARTIFACT_COST + test_constants::MANA_STORED,
        g.opponent.other_player.mana()
    );
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::User.into() }),
        g.client.cards.get(id).position()
    );
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::Opponent.into() }),
        g.opponent.cards.get(id).position()
    );
}

#[test]
fn summon_project_at_end_of_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestProjectTriggeredAbilityTakeManaAtDusk);
    assert_eq!(test_constants::STARTING_MANA, g.client.this_player.mana());
    g.pass_turn(Side::Covenant);
    assert!(g.dawn());
    g.spend_all_action_points(Side::Riftcaller);
    g.opponent_click(Button::EndTurn);
    assert_eq!(test_constants::STARTING_MANA, g.client.this_player.mana());
    g.summon_project(id);
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::SUMMON_PROJECT_COST,
        g.client.this_player.mana()
    );
    g.click(Button::StartTurn);
    assert!(g.dusk());
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::SUMMON_PROJECT_COST
            + test_constants::MANA_TAKEN,
        g.client.this_player.mana()
    );
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::SUMMON_PROJECT_COST
            + test_constants::MANA_TAKEN,
        g.opponent.other_player.mana()
    );
}

#[test]
fn summon_project_during_minion_summon_decision() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    let id = g.create_and_play(CardName::TestSummonboundProject);
    g.pass_turn(Side::Covenant);

    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.client.this_player.mana());
    g.summon_project(id);
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::SUMMON_PROJECT_COST,
        g.client.this_player.mana()
    );
}

#[test]
fn cannot_summon_duskbound_during_minion_summon_decision() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    let id = g.create_and_play(CardName::TestDuskboundProject);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(test_constants::ROOM_ID);
    assert!(g.summon_project_with_result(id).is_err());
}

#[test]
fn summon_project_during_room_approach() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestRoomboundProject);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.client.this_player.mana());
    g.summon_project(id);
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::SUMMON_PROJECT_COST,
        g.client.this_player.mana()
    );
    g.opponent_click(Button::EndRaid);
}

#[test]
fn cannot_summon_summonbound_during_room_approach() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestSummonboundProject);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.client.this_player.mana());
    assert!(g.summon_project_with_result(id).is_err());
}

#[test]
fn cannot_summon_duskbound_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    let id = g.create_and_play(CardName::TestDuskboundProject);
    g.pass_turn(Side::Covenant);

    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.client.this_player.mana());
    assert!(g.summon_project_with_result(id).is_err());
}

#[test]
fn cannot_summon_nightbound_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    let id = g.create_and_play(CardName::TestNightboundProject);
    g.pass_turn(Side::Covenant);

    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.client.this_player.mana());
    assert!(g.summon_project_with_result(id).is_err());
}

#[test]
fn cannot_summon_trap_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    let id = g.create_and_play(CardName::TestTrapProject);
    g.pass_turn(Side::Covenant);

    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(test_constants::STARTING_MANA, g.client.this_player.mana());
    assert!(g.summon_project_with_result(id).is_err());
}

#[test]
fn triggered_ability_cannot_summon_project() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).mana(0)).actions(1).build();
    g.create_and_play(CardName::TestProjectTriggeredAbilityTakeManaAtDusk);
    g.pass_turn(Side::Covenant);
    assert!(g.dawn());
    assert_eq!(0, g.client.this_player.mana());
    g.pass_turn(Side::Riftcaller);
    assert!(g.dusk());
    assert_eq!(0, g.client.this_player.mana());
    assert_eq!(0, g.opponent.other_player.mana());
}

#[test]
fn cannot_summon_duskbound_immediately() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestDuskboundProject);
    assert!(g.summon_project_with_result(id).is_err());
}

#[test]
fn cannot_summon_trap_immediately() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestTrapProject);
    assert!(g.summon_project_with_result(id).is_err());
}

#[test]
fn summon_nightbound_immediately() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestNightboundProject);
    assert!(g.summon_project_with_result(id).is_ok());
}

#[test]
fn summon_duskbound_at_end_of_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestDuskboundProject);
    g.pass_turn(Side::Covenant);
    g.move_to_end_step(Side::Riftcaller);
    assert!(g.dawn()); // Game should pause on end step
    assert!(g.side_has(Button::StartTurn, Side::Covenant));
    assert!(g.summon_project_with_result(id).is_ok());
}

#[test]
fn cannot_summon_nightbound_at_end_of_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::TestNightboundProject);
    g.pass_turn(Side::Covenant);
    g.move_to_end_step(Side::Riftcaller);
    assert!(g.dusk()); // Game should *not* pause on end step
}

#[test]
fn cannot_summon_trap_at_end_of_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::TestTrapProject);
    g.pass_turn(Side::Covenant);
    g.move_to_end_step(Side::Riftcaller);
    assert!(g.dusk()); // Game should *not* pause on end step
}

#[test]
fn triggered_ability_take_all_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).actions(1).build();
    let id = g.create_and_play(CardName::TestProjectTriggeredAbilityTakeManaAtDusk);
    g.pass_turn(Side::Covenant);
    g.move_to_end_step(Side::Riftcaller);
    g.summon_project(id);
    g.click(Button::StartTurn);

    let mut taken = 0;
    while taken < test_constants::MANA_STORED {
        assert!(g.dusk());
        taken += test_constants::MANA_TAKEN;
        g.pass_turn(Side::Covenant);
        assert!(g.dawn());
        g.pass_turn(Side::Riftcaller);
    }

    assert_eq!(
        test_constants::STARTING_MANA - test_constants::SUMMON_PROJECT_COST
            + test_constants::MANA_STORED,
        g.client.this_player.mana()
    );
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::SUMMON_PROJECT_COST
            + test_constants::MANA_STORED,
        g.opponent.other_player.mana()
    );
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::User.into() }),
        g.client.cards.get(id).position()
    );
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::Opponent.into() }),
        g.opponent.cards.get(id).position()
    );
}

#[test]
fn use_artifact_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).current_turn(Side::Covenant).build();
    g.create_and_play(CardName::TestScheme3_10);
    g.create_and_play(CardName::TestMinionEndRaid);
    g.pass_turn(Side::Covenant);
    let id = g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::Summon);
    assert_eq!(g.client.cards.hand().real_cards().len(), 0);
    assert_eq!(g.client.cards.hand().token_cards().len(), 1);
    g.activate_ability(id, 0);
    assert_eq!(g.client.cards.hand().real_cards().len(), 1);
    assert_eq!(g.client.cards.hand().token_cards().len(), 0);
}

#[test]
fn cannot_use_action_artifact_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller)).current_turn(Side::Covenant).build();
    g.create_and_play(CardName::TestScheme3_10);
    g.create_and_play(CardName::TestMinionEndRaid);
    g.pass_turn(Side::Covenant);
    let id = g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::Summon);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn use_project_ability_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestProjectSacrificeToEndRaid);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(test_constants::ROOM_ID);
    g.summon_project(id);
    g.activate_ability(id, 0);
    assert!(!g.client.data.raid_active());
}

#[test]
fn use_project_ability_during_subsequent_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestProjectSacrificeToEndRaid);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(test_constants::ROOM_ID);
    g.summon_project(id);
    g.click(Button::ProceedToAccess);
    g.opponent_click(Button::EndRaid);
    g.initiate_raid(test_constants::ROOM_ID);
    g.activate_ability(id, 0);
    assert!(!g.client.data.raid_active());
}

#[test]
fn cannot_use_project_ability_during_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestProjectSacrificeToEndRaid);
    g.pass_turn(Side::Covenant);
    g.initiate_raid(test_constants::ROOM_ID);
    g.summon_project(id);
    g.click(Button::ProceedToAccess);
    g.opponent_click(Button::EndRaid);
    g.pass_turn(Side::Riftcaller);
    assert!(g.dusk());
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn discard_to_hand_size() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).hand_size(5)).build();
    g.draw_card();
    let discard_id = g.client.cards.hand()[0].id();

    assert_eq!(g.client.cards.hand().real_cards().len(), 6);
    let ids = g.client.cards.hand().real_cards().iter().map(|c| c.id()).collect::<Vec<_>>();
    eprintln!("Hand IDs: {:?}", ids);
    g.move_to_end_step(Side::Covenant);

    g.move_selector_card(discard_id);
    g.click(Button::SubmitDiscard);

    assert_eq!(g.client.cards.hand().real_cards().len(), 5);
    assert_eq!(g.client.cards.discard_pile().len(), 1);
    assert!(g.dawn());
}

#[test]
fn discard_to_hand_size_wounds() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).hand_size(5).wounds(1)).build();
    let discard_id = g.client.cards.hand()[0].id();

    assert_eq!(g.client.cards.hand().real_cards().len(), 5);
    g.move_to_end_step(Side::Riftcaller);

    g.move_selector_card(discard_id);
    g.click(Button::SubmitDiscard);

    assert_eq!(g.client.cards.hand().real_cards().len(), 4);
    assert_eq!(g.client.cards.discard_pile().len(), 1);
    assert!(g.dusk());
}

#[test]
fn cannot_discard_extra_to_hand_size() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).hand_size(5)).build();
    g.draw_card();
    let hand = g.client.cards.hand();
    let d1 = hand[0].id();
    let d2 = hand[1].id();

    assert_eq!(g.client.cards.hand().real_cards().len(), 6);
    g.move_to_end_step(Side::Covenant);

    g.move_selector_card(d1);
    g.move_selector_card(d2);
    assert!(g.click_with_result(Button::SubmitDiscard).is_err());
}

#[test]
fn cannot_discard_too_few_to_hand_size() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).hand_size(5)).build();
    g.draw_card();
    assert_eq!(g.client.cards.hand().real_cards().len(), 6);
    g.move_to_end_step(Side::Covenant);
    assert!(g.click_with_result(Button::SubmitDiscard).is_err());
}

#[test]
fn remove_curse() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).curses(1)).build();
    assert_eq!(g.client.cards.hand().token_cards().names(), vec!["Curse"]);
    assert_eq!(g.opponent.cards.opponent_hand().token_cards().names(), vec!["Curse"]);
    let card_id = g.client.cards.hand().token_cards()[0].id();
    g.perform(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    );
    assert_eq!(test_constants::STARTING_MANA - 2, g.me().mana());
    assert!(g.client.cards.hand().token_cards().is_empty());
}

#[test]
fn cannot_remove_curse_without_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).curses(1).mana(0)).build();
    let card_id = g.client.cards.hand().token_cards()[0].id();
    assert!(g
        .perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
            g.user_id(),
        )
        .is_err());
}

#[test]
fn cannot_remove_curse_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Riftcaller).curses(1)).build();
    let card_id = g.client.cards.hand().token_cards()[0].id();
    g.pass_turn(Side::Riftcaller);
    assert!(g
        .perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
            g.user_id(),
        )
        .is_err());
}

#[test]
fn destroy_evocation_while_cursed() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant))
        .opponent(TestSide::new(Side::Riftcaller).curses(1))
        .build();
    g.pass_turn(Side::Covenant);
    let evocation_id = g.create_and_play(CardName::TestEvocation);
    g.pass_turn(Side::Riftcaller);
    assert_eq!(g.client.cards.hand().token_cards().names(), vec!["Dispel Evocation"]);
    assert_eq!(g.opponent.cards.opponent_hand().token_cards().names(), vec!["Dispel Evocation"]);
    let dispel_card_id = g.client.cards.hand().token_cards()[0].id();
    g.perform(
        Action::PlayCard(PlayCardAction { card_id: Some(dispel_card_id), target: None }),
        g.user_id(),
    );
    g.click(Button::Destroy);
    assert_eq!(test_constants::STARTING_MANA - 2, g.me().mana());
    assert_eq!(g.client.cards.opponent_discard_pile().ids(), vec![evocation_id])
}

#[test]
fn cannot_destroy_evocation_without_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).mana(0))
        .opponent(TestSide::new(Side::Riftcaller).curses(1))
        .build();
    g.pass_turn(Side::Covenant);
    g.create_and_play(CardName::TestEvocation);
    g.pass_turn(Side::Riftcaller);
    let dispel_card_id = g.client.cards.hand().token_cards()[0].id();
    assert!(g
        .perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(dispel_card_id), target: None }),
            g.user_id(),
        )
        .is_err());
}

#[test]
fn cannot_destroy_evocation_without_targets() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant))
        .opponent(TestSide::new(Side::Riftcaller).curses(1))
        .build();
    let dispel_card_id = g.client.cards.hand().token_cards()[0].id();
    assert!(g
        .perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(dispel_card_id), target: None }),
            g.user_id(),
        )
        .is_err());
}

#[test]
fn cannot_destroy_evocation_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant).mana(0))
        .opponent(TestSide::new(Side::Riftcaller).curses(1))
        .build();
    g.pass_turn(Side::Covenant);
    g.create_and_play(CardName::TestEvocation);
    let dispel_card_id = g.client.cards.hand().token_cards()[0].id();
    assert!(g
        .perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(dispel_card_id), target: None }),
            g.user_id(),
        )
        .is_err());
}

#[test]
fn legal_actions() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    assert!(g.legal_actions_result(Side::Riftcaller).is_err());
    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Covenant),
        vec![GameAction::GainMana, GameAction::DrawCard],
    );

    let spell_id = test_helpers::server_card_id(g.add_to_hand(CardName::TestRitual));

    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Covenant),
        vec![
            GameAction::GainMana,
            GameAction::DrawCard,
            GameAction::PlayCard(spell_id, game_actions::CardTarget::None),
        ],
    );

    let minion_id = test_helpers::server_card_id(g.add_to_hand(CardName::TestMinionEndRaid));

    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Covenant),
        vec![
            GameAction::GainMana,
            GameAction::DrawCard,
            GameAction::PlayCard(spell_id, game_actions::CardTarget::None),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::Sanctum)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::Vault)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::Crypt)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::RoomA)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::RoomB)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::RoomC)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::RoomD)),
            GameAction::PlayCard(minion_id, game_actions::CardTarget::Room(RoomId::RoomE)),
        ],
    );
}

#[test]
fn legal_actions_progress_room() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    g.create_and_play(CardName::TestScheme3_10);
    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Covenant),
        vec![GameAction::GainMana, GameAction::DrawCard, GameAction::ProgressRoom(RoomId::RoomA)],
    );
}

#[test]
fn riftcaller_legal_actions() {
    let g = TestGame::new(TestSide::new(Side::Riftcaller)).build();
    assert!(g.legal_actions_result(Side::Covenant).is_err());
    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Riftcaller),
        vec![
            GameAction::GainMana,
            GameAction::DrawCard,
            GameAction::InitiateRaid(RoomId::Sanctum),
            GameAction::InitiateRaid(RoomId::Vault),
            GameAction::InitiateRaid(RoomId::Crypt),
        ],
    );
}
