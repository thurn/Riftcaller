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
    test_helpers::assert_cards_match(
        g.user.cards.hand(),
        vec![CardName::TestMinionDealDamageEndRaid],
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

    test_helpers::assert_cards_match(
        g.user.cards.hand(),
        vec![CardName::TestMinionDealDamageEndRaid],
    );
    assert_eq!(vec![test_constants::HIDDEN_CARD], g.opponent.cards.opponent_hand().names());
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
fn play_card() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).mana(5)).build();
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
    test_helpers::assert_cards_match(g.user.cards.discard_pile(), vec![CardName::ArcaneRecovery]);
    test_helpers::assert_cards_match(
        g.opponent.cards.opponent_discard_pile(),
        vec![CardName::ArcaneRecovery],
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
    test_helpers::assert_cards_match(
        g.user.cards.room_occupants(test_constants::ROOM_ID),
        vec![CardName::GoldMine],
    );
    assert_eq!(
        vec![test_constants::HIDDEN_CARD],
        g.opponent.cards.room_occupants(test_constants::ROOM_ID).names()
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
fn progress_room() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(10)).build();
    g.create_and_play(CardName::TestScheme3_10);
    let response = g.perform_action(
        Action::ProgressRoom(ProgressRoomAction { room_id: test_constants::CLIENT_ROOM_ID.into() }),
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
    g.create_and_play(CardName::TestAstralMinion);
    g.create_and_play(CardName::TestMortalMinion);
    assert_eq!(g.user.cards.room_defenders(test_constants::ROOM_ID).len(), 4);
    g.create_and_play(CardName::TestMinionDealDamage);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.room_defenders(test_constants::ROOM_ID).len(), 4);
    assert_eq!(g.user.cards.discard_pile().len(), 1);
}

#[test]
fn minion_limit_cannot_take_other_actions() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(6).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TestInfernalMinion);
    g.create_and_play(CardName::TestAstralMinion);
    g.create_and_play(CardName::TestMortalMinion);
    g.create_and_play(CardName::TestMinionDealDamage);
    assert!(g.draw_card_with_result().is_err());
}

#[test]
fn minion_limit_cancel_playing() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(6).build();
    g.create_and_play(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TestInfernalMinion);
    g.create_and_play(CardName::TestAstralMinion);
    g.create_and_play(CardName::TestMortalMinion);
    g.create_and_play(CardName::TestMinionDealDamage);
    g.click(Button::CancelPlayingCard);
    assert_eq!(g.user.cards.room_defenders(test_constants::ROOM_ID).len(), 4);
    assert_eq!(g.user.cards.hand().real_cards().len(), 1);
    assert!(g.draw_card_with_result().is_ok());
}

#[test]
fn weapon_limit() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).actions(6).build();
    g.create_and_play(CardName::TestWeapon2Attack);
    g.create_and_play(CardName::TestWeapon2Attack);
    g.create_and_play(CardName::TestWeapon2Attack);
    g.create_and_play(CardName::TestWeapon2Attack);
    g.create_and_play(CardName::TestWeapon2Attack);
    assert_eq!(g.user.cards.discard_pile().len(), 0);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.discard_pile().len(), 1);
}

#[test]
fn evocation_limit() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).actions(6).build();
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    assert_eq!(g.user.cards.discard_pile().len(), 0);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.discard_pile().len(), 1);
}

#[test]
fn sacrifice_existing_project() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestScheme3_10);
    g.create_and_play(CardName::TestProject2Cost3Raze);
    assert_eq!(g.user.cards.discard_pile().len(), 0);
    g.click(Button::Sacrifice);
    assert_eq!(g.user.cards.discard_pile().names(), vec!["Test Scheme 3_10"]);
}

#[test]
fn score_overlord_card() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(10)).actions(5).build();
    let scheme_id = g.create_and_play(CardName::TestScheme3_10);
    let progress =
        Action::ProgressRoom(ProgressRoomAction { room_id: test_constants::CLIENT_ROOM_ID.into() });
    g.perform(progress.clone(), g.user_id());
    g.perform(progress.clone(), g.user_id());
    let response = g.perform_action(progress, g.user_id());

    assert_snapshot!(Summary::run(&response));
    assert!(g.opponent.cards.get(scheme_id).revealed_to_me());
    assert_eq!(g.user.this_player.mana(), 7);
    assert_eq!(g.opponent.other_player.mana(), 7);
    assert_eq!(g.user.this_player.score(), 10);
    assert_eq!(g.opponent.other_player.score(), 10);
}

#[test]
fn overlord_win_game() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).mana(10).bonus_points(90)).actions(5).build();
    g.create_and_play(CardName::TestScheme3_10);
    let progress =
        Action::ProgressRoom(ProgressRoomAction { room_id: test_constants::CLIENT_ROOM_ID.into() });
    g.perform(progress.clone(), g.user_id());
    g.perform(progress.clone(), g.user_id());
    let response = g.perform_action(progress, g.user_id());

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
    let mut g = TestGame::new(TestSide::new(Side::Champion)).actions(3).build();
    let id = g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    let ability_card_id = g
        .user
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
    let id = g.create_and_play(CardName::TestProjectTriggeredAbilityTakeManaAtDusk);
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
    g.create_and_play(CardName::TestProjectTriggeredAbilityTakeManaAtDusk);
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
    g.move_to_end_step(Side::Champion);
    assert!(g.dawn()); // Game should pause on end step
    assert!(g.side_has(Button::StartTurn, Side::Overlord));
    assert!(g.unveil_card_with_result(id).is_ok());
}

#[test]
fn cannot_unveil_nightbound_at_end_of_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestNightboundProject);
    g.pass_turn(Side::Overlord);
    g.move_to_end_step(Side::Champion);
    assert!(g.dusk()); // Game should *not* pause on end step
}

#[test]
fn cannot_unveil_trap_at_end_of_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestTrapProject);
    g.pass_turn(Side::Overlord);
    g.move_to_end_step(Side::Champion);
    assert!(g.dusk()); // Game should *not* pause on end step
}

#[test]
fn triggered_ability_take_all_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(1).build();
    let id = g.create_and_play(CardName::TestProjectTriggeredAbilityTakeManaAtDusk);
    g.pass_turn(Side::Overlord);
    g.move_to_end_step(Side::Champion);
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
    g.create_and_play(CardName::TestScheme3_10);
    g.create_and_play(CardName::TestMinionEndRaid);
    g.pass_turn(Side::Overlord);
    let id = g.create_and_play(CardName::TestSacrificeDrawCardArtifact);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::Summon);
    assert_eq!(g.user.cards.hand().real_cards().len(), 0);
    assert_eq!(g.user.cards.hand().token_cards().len(), 1);
    g.activate_ability(id, 0);
    assert_eq!(g.user.cards.hand().real_cards().len(), 1);
    assert_eq!(g.user.cards.hand().token_cards().len(), 0);
}

#[test]
fn cannot_use_action_artifact_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).current_turn(Side::Overlord).build();
    g.create_and_play(CardName::TestScheme3_10);
    g.create_and_play(CardName::TestMinionEndRaid);
    g.pass_turn(Side::Overlord);
    let id = g.create_and_play(CardName::TestActivatedAbilityTakeMana);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::Summon);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn use_project_ability_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestProjectSacrificeToEndRaid);
    g.pass_turn(Side::Overlord);
    g.initiate_raid(test_constants::ROOM_ID);
    g.unveil_card(id);
    g.activate_ability(id, 0);
    assert!(!g.user.data.raid_active());
}

#[test]
fn use_project_ability_during_subsequent_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestProjectSacrificeToEndRaid);
    g.pass_turn(Side::Overlord);
    g.initiate_raid(test_constants::ROOM_ID);
    g.unveil_card(id);
    g.click(Button::ProceedToAccess);
    g.opponent_click(Button::EndRaid);
    g.initiate_raid(test_constants::ROOM_ID);
    g.activate_ability(id, 0);
    assert!(!g.user.data.raid_active());
}

#[test]
fn cannot_use_project_ability_during_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestProjectSacrificeToEndRaid);
    g.pass_turn(Side::Overlord);
    g.initiate_raid(test_constants::ROOM_ID);
    g.unveil_card(id);
    g.click(Button::ProceedToAccess);
    g.opponent_click(Button::EndRaid);
    g.pass_turn(Side::Champion);
    assert!(g.dusk());
    assert!(g.activate_ability_with_result(id, 0).is_err());
}

#[test]
fn discard_to_hand_size() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).hand_size(5)).build();
    g.draw_card();
    let discard_id = g.user.cards.hand()[0].id();

    assert_eq!(g.user.cards.hand().real_cards().len(), 6);
    g.move_to_end_step(Side::Overlord);

    g.move_selector_card(discard_id);
    g.click(Button::SubmitDiscard);

    assert_eq!(g.user.cards.hand().real_cards().len(), 5);
    assert_eq!(g.user.cards.discard_pile().len(), 1);
    assert!(g.dawn());
}

#[test]
fn discard_to_hand_size_wounds() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).hand_size(5).wounds(1)).build();
    let discard_id = g.user.cards.hand()[0].id();

    assert_eq!(g.user.cards.hand().real_cards().len(), 5);
    g.move_to_end_step(Side::Champion);

    g.move_selector_card(discard_id);
    g.click(Button::SubmitDiscard);

    assert_eq!(g.user.cards.hand().real_cards().len(), 4);
    assert_eq!(g.user.cards.discard_pile().len(), 1);
    assert!(g.dusk());
}

#[test]
fn cannot_discard_extra_to_hand_size() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).hand_size(5)).build();
    g.draw_card();
    let hand = g.user.cards.hand();
    let d1 = hand[0].id();
    let d2 = hand[1].id();

    assert_eq!(g.user.cards.hand().real_cards().len(), 6);
    g.move_to_end_step(Side::Overlord);

    g.move_selector_card(d1);
    g.move_selector_card(d2);
    assert!(g.click_with_result(Button::SubmitDiscard).is_err());
}

#[test]
fn cannot_discard_too_few_to_hand_size() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).hand_size(5)).build();
    g.draw_card();
    assert_eq!(g.user.cards.hand().real_cards().len(), 6);
    g.move_to_end_step(Side::Overlord);
    assert!(g.click_with_result(Button::SubmitDiscard).is_err());
}

#[test]
fn undo_gain_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.gain_mana();
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA + 1);
    assert_eq!(g.me().actions(), 3);
    g.click(Button::Undo);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA);
    assert_eq!(g.me().actions(), 4);
}

#[test]
fn undo_play_card() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::TestWeaponAbyssal);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - test_constants::WEAPON_COST);
    assert_eq!(g.me().actions(), 3);
    g.click(Button::Undo);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA);
    assert_eq!(g.me().actions(), 4);
}

#[test]
fn cannot_undo_draw_card() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.draw_card();
    assert!(g.click_with_result(Button::Undo).is_err());
}

#[test]
fn remove_curse() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).curses(1)).build();
    assert_eq!(g.user.cards.hand().token_cards().names(), vec!["Curse"]);
    assert_eq!(g.opponent.cards.opponent_hand().token_cards().names(), vec!["Curse"]);
    let card_id = g.user.cards.hand().token_cards()[0].id();
    g.perform(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    );
    assert_eq!(test_constants::STARTING_MANA - 2, g.me().mana());
    assert!(g.user.cards.hand().token_cards().is_empty());
}

#[test]
fn cannot_remove_curse_without_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).curses(1).mana(0)).build();
    let card_id = g.user.cards.hand().token_cards()[0].id();
    assert!(g
        .perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
            g.user_id(),
        )
        .is_err());
}

#[test]
fn cannot_remove_curse_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).curses(1)).build();
    let card_id = g.user.cards.hand().token_cards()[0].id();
    g.pass_turn(Side::Champion);
    assert!(g
        .perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
            g.user_id(),
        )
        .is_err());
}

#[test]
fn destroy_evocation_while_cursed() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).curses(1))
        .build();
    g.pass_turn(Side::Overlord);
    let evocation_id = g.create_and_play(CardName::TestEvocation);
    g.pass_turn(Side::Champion);
    assert_eq!(g.user.cards.hand().token_cards().names(), vec!["Dispel Evocation"]);
    assert_eq!(g.opponent.cards.opponent_hand().token_cards().names(), vec!["Dispel Evocation"]);
    let dispel_card_id = g.user.cards.hand().token_cards()[0].id();
    g.perform(
        Action::PlayCard(PlayCardAction { card_id: Some(dispel_card_id), target: None }),
        g.user_id(),
    );
    g.click(Button::Destroy);
    assert_eq!(test_constants::STARTING_MANA - 2, g.me().mana());
    assert_eq!(g.user.cards.opponent_discard_pile().ids(), vec![evocation_id])
}

#[test]
fn cannot_destroy_evocation_without_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(0))
        .opponent(TestSide::new(Side::Champion).curses(1))
        .build();
    g.pass_turn(Side::Overlord);
    g.create_and_play(CardName::TestEvocation);
    g.pass_turn(Side::Champion);
    let dispel_card_id = g.user.cards.hand().token_cards()[0].id();
    assert!(g
        .perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(dispel_card_id), target: None }),
            g.user_id(),
        )
        .is_err());
}

#[test]
fn cannot_destroy_evocation_without_targets() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).curses(1))
        .build();
    let dispel_card_id = g.user.cards.hand().token_cards()[0].id();
    assert!(g
        .perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(dispel_card_id), target: None }),
            g.user_id(),
        )
        .is_err());
}

#[test]
fn cannot_destroy_evocation_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(0))
        .opponent(TestSide::new(Side::Champion).curses(1))
        .build();
    g.pass_turn(Side::Overlord);
    g.create_and_play(CardName::TestEvocation);
    let dispel_card_id = g.user.cards.hand().token_cards()[0].id();
    assert!(g
        .perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(dispel_card_id), target: None }),
            g.user_id(),
        )
        .is_err());
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
fn legal_actions_progress_room() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TestScheme3_10);
    test_helpers::assert_contents_equal(
        g.legal_actions(Side::Overlord),
        vec![GameAction::GainMana, GameAction::DrawCard, GameAction::ProgressRoom(RoomId::RoomA)],
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
