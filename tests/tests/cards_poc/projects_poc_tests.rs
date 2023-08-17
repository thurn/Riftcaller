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

use cards_test::test_cards::{MANA_STORED, MANA_TAKEN, UNVEIL_COST};
use game_data::card_name::CardName;
use game_data::primitives::Side;
use protos::spelldawn::PlayerName;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn test_card_stored_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestTriggeredAbilityTakeManaAtDusk);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    assert!(g.dawn());
    assert_eq!(STARTING_MANA, g.me().mana());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    click_on_unveil(&mut g);
    assert!(g.dusk());
    assert_eq!(STARTING_MANA - UNVEIL_COST + MANA_TAKEN, g.me().mana());
    assert_eq!((MANA_STORED - MANA_TAKEN).to_string(), g.user.get_card(id).arena_icon());
}

#[test]
fn gemcarver() {
    let (card_cost, taken) = (2, 3);
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::Gemcarver);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    click_on_unveil(&mut g);
    assert_eq!(STARTING_MANA - card_cost + taken, g.me().mana());
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(STARTING_MANA - card_cost + taken * 2, g.me().mana());
    assert_eq!(2, g.user.cards.hand(PlayerName::User).len());
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(STARTING_MANA - card_cost + taken * 3, g.me().mana());
    assert_eq!(4, g.user.cards.hand(PlayerName::User).len());
}

#[test]
fn spike_trap() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(5))
        .build();
    g.create_and_play(CardName::SpikeTrap);
    level_up_room(&mut g, 2);
    assert!(g.dawn());
    assert_eq!(6, g.user.cards.hand(PlayerName::Opponent).len());
    g.initiate_raid(ROOM_ID);
    assert_eq!(2, g.user.cards.hand(PlayerName::Opponent).len());
}

#[test]
fn spike_trap_no_counters() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(5))
        .build();
    g.create_and_play(CardName::SpikeTrap);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    assert!(g.dawn());
    assert_eq!(6, g.user.cards.hand(PlayerName::Opponent).len());
    g.initiate_raid(ROOM_ID);
    assert_eq!(4, g.user.cards.hand(PlayerName::Opponent).len());
}

#[test]
fn spike_trap_victory() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(0))
        .build();
    g.create_and_play(CardName::SpikeTrap);
    level_up_room(&mut g, 2);
    assert!(g.dawn());
    g.initiate_raid(ROOM_ID);
    assert!(g.is_victory_for_player(Side::Overlord));
}
