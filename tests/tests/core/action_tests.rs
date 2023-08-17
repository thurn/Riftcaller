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

use cards_test::test_cards::{ARTIFACT_COST, MANA_STORED, MANA_TAKEN, UNVEIL_COST};
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
use test_utils::*;

#[test]
fn connect() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).connect(false).build();
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
    assert_ok(&r1);
    let r2 = g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id());
    assert_identical(
        vec![CardName::TestMinionDealDamageEndRaid],
        g.user.cards.hand(PlayerName::User),
    );
    assert_ok(&r2);
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

    assert_identical(
        vec![CardName::TestMinionDealDamageEndRaid],
        g.user.cards.hand(PlayerName::User),
    );
    assert_eq!(vec![HIDDEN_CARD], g.opponent.cards.hand(PlayerName::Opponent));
    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
}

#[test]
fn cannot_draw_card_on_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.opponent_id()));
}

#[test]
fn cannot_draw_when_out_of_action_points() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(0).build();
    assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id()));
}

#[test]
fn cannot_draw_during_raid() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord)).actions(0).raid(TestRaid::new()).build();
    assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id()));
}

#[test]
fn maximum_hand_size() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).deck_top(CardName::TestMinionEndRaid)).build();
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(4, g.user.cards.hand(PlayerName::User).len());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(8, g.user.cards.hand(PlayerName::User).len());
    spend_actions_until_turn_over(&mut g, Side::Overlord);
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
    assert_identical(vec![CardName::ArcaneRecovery], g.user.cards.discard_pile(PlayerName::User));
    assert_identical(
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
                card_target: Some(card_target::CardTarget::RoomId(CLIENT_ROOM_ID.into())),
            }),
        }),
        g.user_id(),
    );
    assert_snapshot!(Summary::run(&response));

    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(0, g.me().mana());
    assert_eq!(0, g.opponent.other_player.mana());
    assert_identical(
        vec![CardName::GoldMine],
        g.user.cards.room_cards(ROOM_ID, ClientRoomLocation::Back),
    );
    assert_eq!(vec![HIDDEN_CARD], g.opponent.cards.room_cards(ROOM_ID, ClientRoomLocation::Back));
}

#[test]
fn cannot_play_card_on_opponent_turn() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn cannot_play_card_when_out_of_action_points() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).actions(0).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn cannot_play_card_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).raid(TestRaid::new()).build();
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    assert_error(g.perform_action(
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
    assert_error(g.perform_action(Action::GainMana(GainManaAction {}), g.opponent_id()));
}

#[test]
fn cannot_gain_mana_when_out_of_action_points() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(0).build();
    assert_error(g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()));
}

#[test]
fn cannot_gain_mana_during_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).raid(TestRaid::new()).build();
    assert_error(g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()));
}

#[test]
fn level_up_room() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(10)).build();
    g.create_and_play(CardName::TestScheme3_15);
    let response = g.perform_action(
        Action::LevelUpRoom(LevelUpRoomAction { room_id: CLIENT_ROOM_ID.into() }),
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
    assert_eq!(g.user.cards.room_cards(ROOM_ID, ClientRoomLocation::Front).len(), 4);
    g.create_and_play(CardName::TestMinionDealDamage);
    assert_eq!(g.user.cards.room_cards(ROOM_ID, ClientRoomLocation::Front).len(), 4);
    assert_eq!(g.user.cards.discard_pile(PlayerName::User), vec!["Test Minion End Raid"]);
}

#[test]
fn score_overlord_card() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(10)).actions(5).build();
    let scheme_id = g.create_and_play(CardName::TestScheme3_15);
    let level_up = Action::LevelUpRoom(LevelUpRoomAction { room_id: CLIENT_ROOM_ID.into() });
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
    let level_up = Action::LevelUpRoom(LevelUpRoomAction { room_id: CLIENT_ROOM_ID.into() });
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
    g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()).unwrap();
    g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()).unwrap();
    let response = g.perform_action(Action::GainMana(GainManaAction {}), g.user_id());
    assert_snapshot!(Summary::run(&response));

    assert_eq!(8, g.me().mana());
    assert_eq!(8, g.opponent.other_player.mana());
    assert_eq!(0, g.me().actions());
    assert_eq!(0, g.opponent.other_player.actions());
    assert_eq!(3, g.user.other_player.actions());
    assert_eq!(3, g.opponent.this_player.actions());
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

    assert_eq!(STARTING_MANA - ARTIFACT_COST, g.me().mana());
    assert_eq!(2, g.me().actions());

    let response = g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(ability_card_id), target: None }),
        g.user_id(),
    );

    assert_snapshot!(Summary::run(&response));
    assert_eq!(STARTING_MANA - ARTIFACT_COST + MANA_TAKEN, g.me().mana());
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
    while taken < MANA_STORED {
        g.perform(
            Action::PlayCard(PlayCardAction { card_id: Some(ability_card_id), target: None }),
            g.user_id(),
        );
        taken += MANA_TAKEN;

        spend_actions_until_turn_over(&mut g, Side::Champion);
        assert!(g.dusk());
        spend_actions_until_turn_over(&mut g, Side::Overlord);
        assert!(g.dawn());
    }

    assert_eq!(STARTING_MANA - ARTIFACT_COST + MANA_STORED, g.user.this_player.mana());
    assert_eq!(STARTING_MANA - ARTIFACT_COST + MANA_STORED, g.opponent.other_player.mana());
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
fn triggered_unveil_ability() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(1).build();
    g.create_and_play(CardName::TestTriggeredAbilityTakeManaAtDusk);
    assert!(g.dawn());
    assert_eq!(STARTING_MANA, g.user.this_player.mana());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert!(g.dusk());
    click_on_unveil(&mut g);
    assert_eq!(STARTING_MANA - UNVEIL_COST + MANA_TAKEN, g.user.this_player.mana());
    assert_eq!(STARTING_MANA - UNVEIL_COST + MANA_TAKEN, g.opponent.other_player.mana());
}

#[test]
fn triggered_ability_cannot_unveil() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord).mana(0)).actions(1).build();
    g.create_and_play(CardName::TestTriggeredAbilityTakeManaAtDusk);
    assert!(g.dawn());
    assert_eq!(0, g.user.this_player.mana());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert!(g.dusk());
    assert_eq!(0, g.user.this_player.mana());
    assert_eq!(0, g.opponent.other_player.mana());
}

#[test]
fn triggered_ability_take_all_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).actions(1).build();
    let id = g.create_and_play(CardName::TestTriggeredAbilityTakeManaAtDusk);
    let mut taken = 0;
    let mut unveiled = false;
    while taken < MANA_STORED {
        assert!(g.dawn());
        spend_actions_until_turn_over(&mut g, Side::Champion);
        assert!(g.dusk());
        if !unveiled {
            click_on_unveil(&mut g);
            unveiled = true;
        }
        taken += MANA_TAKEN;
        spend_actions_until_turn_over(&mut g, Side::Overlord);
    }

    assert_eq!(STARTING_MANA - UNVEIL_COST + MANA_STORED, g.user.this_player.mana());
    assert_eq!(STARTING_MANA - UNVEIL_COST + MANA_STORED, g.opponent.other_player.mana());
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
fn legal_actions() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    assert!(g.legal_actions_result(Side::Champion).is_err());
    assert_contents_equal(
        g.legal_actions(Side::Overlord),
        vec![GameAction::GainMana, GameAction::DrawCard],
    );

    let spell_id = server_card_id(g.add_to_hand(CardName::TestOverlordSpell));

    assert_contents_equal(
        g.legal_actions(Side::Overlord),
        vec![
            GameAction::GainMana,
            GameAction::DrawCard,
            GameAction::PlayCard(spell_id, game_actions::CardTarget::None),
        ],
    );

    let minion_id = server_card_id(g.add_to_hand(CardName::TestMinionEndRaid));

    assert_contents_equal(
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
    assert_contents_equal(
        g.legal_actions(Side::Overlord),
        vec![GameAction::GainMana, GameAction::DrawCard, GameAction::LevelUpRoom(RoomId::RoomA)],
    );
}

#[test]
fn champion_legal_actions() {
    let g = TestGame::new(TestSide::new(Side::Champion)).build();
    assert!(g.legal_actions_result(Side::Overlord).is_err());
    assert_contents_equal(
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
