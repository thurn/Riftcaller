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

use core_ui::icons;
use game_data::card_name::CardName;
use game_data::primitives::{RoomId, Side};
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{ClientRoomLocation, ObjectPositionRaid, PlayerName};
use test_utils::client_interface::HasText;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn test_minion_deal_damage_end_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(5))
        .build();
    g.create_and_play(CardName::TestMinionDealDamageEndRaid);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    assert!(!g.user.data.raid_active());
    assert_eq!(1, g.user.cards.discard_pile(PlayerName::Opponent).len());
    assert_eq!(4, g.user.cards.hand(PlayerName::Opponent).len());
}

#[test]
fn time_golem_pay_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TimeGolem);
    g.set_up_minion_combat();
    assert!(g.opponent.interface.controls().has_text("End Raid"));
    assert!(g.opponent.interface.controls().has_text(format!("Pay 5{}", icons::MANA)));
    assert!(g.opponent.interface.controls().has_text(format!("Pay 2{}", icons::ACTION)));
    g.click_on(g.opponent_id(), format!("Pay 5{}", icons::MANA));
    assert!(g.opponent.interface.controls().has_text("Continue"));
    assert_eq!(test_constants::STARTING_MANA - 5, g.opponent.this_player.mana());
}

#[test]
fn time_golem_defeat() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TimeGolem);
    g.create_and_play(CardName::TestScheme3_10);
    g.pass_turn(Side::Overlord);
    g.create_and_play(CardName::TestWeapon5AttackInfernal);
    g.initiate_raid(test_constants::ROOM_ID);
    g.click(Button::Summon);
    g.click_on(g.opponent_id(), format!("Pay 5{}", icons::MANA));
    g.click_on(g.opponent_id(), "Test Weapon");
    assert_eq!(vec!["Time Golem"], g.user.cards.discard_pile(PlayerName::User));
}

#[test]
fn time_golem_pay_actions() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TimeGolem);
    g.set_up_minion_combat();
    g.click_on(g.opponent_id(), format!("Pay 2{}", icons::ACTION));
    assert_eq!(1, g.opponent.this_player.actions());
    g.opponent_click(Button::NoWeapon);
    g.opponent_click(Button::Score);
    g.opponent_click(Button::EndRaid);
}

#[test]
fn time_golem_end_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::TimeGolem);
    g.set_up_minion_combat();
    g.click_on(g.opponent_id(), "End Raid");
    assert_eq!(3, g.opponent.this_player.actions());
    assert!(!g.user.data.raid_active());
}

#[test]
fn temporal_stalker_end_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.add_to_hand(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TemporalStalker);
    g.set_up_minion_combat();
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    g.opponent_click(Button::NoWeapon);
    g.click_on(g.opponent_id(), "End Raid");
    assert!(!g.user.data.raid_active());
    assert_eq!(
        vec!["Temporal Stalker", "Test Minion End Raid"],
        g.user.cards.room_cards(test_constants::ROOM_ID, ClientRoomLocation::Front)
    );
    assert_eq!(0, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(3, g.opponent.this_player.actions());
}

#[test]
fn temporal_stalker_pay_actions() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.add_to_hand(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TemporalStalker);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    g.click_on(g.opponent_id(), format!("Pay 2{}", icons::ACTION));
    assert_eq!(1, g.opponent.this_player.actions());
    assert!(g.user.data.raid_active());
    assert_eq!(
        vec!["Test Minion End Raid", "Test Scheme 3_10"],
        g.user.cards.names_in_position(Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        vec!["Temporal Stalker"],
        g.user.cards.room_cards(test_constants::ROOM_ID, ClientRoomLocation::Front)
    );
    assert_eq!(0, g.user.cards.hand(PlayerName::User).len());
    assert!(g.opponent.interface.controls().has_text("Continue"));
}

#[test]
fn temporal_stalker_defeat() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.add_to_hand(CardName::TestMinionEndRaid);
    g.create_and_play(CardName::TemporalStalker);
    g.set_up_minion_combat_with_action(|g| {
        g.create_and_play(CardName::TestWeaponAbyssal);
    });
    g.click_on(g.opponent_id(), "Test Weapon");
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(
        vec!["Temporal Stalker"],
        g.user.cards.room_cards(test_constants::ROOM_ID, ClientRoomLocation::Front)
    );
    assert!(g.opponent.interface.controls().has_text("Score"));
}

#[test]
fn shadow_lurker_outer_room() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.add_to_hand(CardName::ShadowLurker);
    assert_eq!("2", g.user.get_card(id).bottom_right_icon());
    let id = g.create_and_play(CardName::ShadowLurker);
    assert_eq!("4", g.user.get_card(id).bottom_right_icon());
    g.set_up_minion_combat_with_action(|g| {
        g.create_and_play(CardName::TestWeaponAbyssal);
    });
    g.click_on(g.opponent_id(), "Test Weapon");
    assert_eq!(test_constants::STARTING_MANA - 5, g.opponent.this_player.mana());
}

#[test]
fn shadow_lurker_inner_room() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.play_with_target_room(CardName::ShadowLurker, RoomId::Sanctum);
    assert_eq!("2", g.user.get_card(id).bottom_right_icon());
}

#[test]
fn sphinx_of_winters_breath_discard_even() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).deck_top(CardName::Test0CostChampionSpell))
        .build();

    g.create_and_play(CardName::SphinxOfWintersBreath);
    g.set_up_minion_combat_with_action(|g| {
        g.add_to_hand(CardName::Test0CostChampionSpell);
    });
    g.opponent_click(Button::NoWeapon);
    assert_eq!(vec!["Test 0 Cost Champion Spell"], g.opponent.cards.discard_pile(PlayerName::User));
    assert!(g.user.data.raid_active());
}

#[test]
fn sphinx_of_winters_breath_discard_odd() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).deck_top(CardName::Test1CostChampionSpell))
        .build();

    g.create_and_play(CardName::SphinxOfWintersBreath);
    g.set_up_minion_combat_with_action(|g| {
        g.add_to_hand(CardName::Test1CostChampionSpell);
    });
    g.opponent_click(Button::NoWeapon);
    assert_eq!(vec!["Test 1 Cost Champion Spell"], g.opponent.cards.discard_pile(PlayerName::User));
    assert!(!g.user.data.raid_active());
}

#[test]
fn bridge_troll_continue() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::BridgeTroll);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    assert!(g.user.data.raid_active());
    assert_eq!(test_constants::STARTING_MANA - 3, g.opponent.this_player.mana());
}

#[test]
fn bridge_troll_end_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).mana(2))
        .build();
    g.create_and_play(CardName::BridgeTroll);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    assert!(!g.user.data.raid_active());
    assert_eq!(0, g.opponent.this_player.mana());
}

#[test]
fn stormcaller_take_2() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(5))
        .build();
    g.create_and_play(CardName::Stormcaller);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    g.click_on(g.opponent_id(), "End Raid");
    assert!(!g.user.data.raid_active());
    assert_eq!(2, g.opponent.cards.discard_pile(PlayerName::User).len());
}

#[test]
fn stormcaller_take_4() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(5))
        .build();
    g.create_and_play(CardName::Stormcaller);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    g.click_on(g.opponent_id(), "Take 2");
    assert!(g.user.data.raid_active());
    assert_eq!(4, g.opponent.cards.discard_pile(PlayerName::User).len());
}

#[test]
fn stormcaller_take_2_game_over() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(0))
        .build();
    g.create_and_play(CardName::Stormcaller);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    assert!(g.is_victory_for_player(Side::Overlord));
}
