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

use data::card_name::CardName;
use data::primitives::Side;
use protos::spelldawn::PlayerName;
use test_utils::*;

#[test]
fn conspire() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Conspire);
    level_up_room(&mut g, 3);
    assert_eq!(g.me().score(), 15);
}

#[test]
fn devise() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Devise);
    level_up_room(&mut g, 4);
    assert_eq!(g.me().score(), 30);
}

#[test]
fn machinate() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Machinate);
    level_up_room(&mut g, 5);
    assert_eq!(g.me().score(), 45);
}

#[test]
fn gathering_dark() {
    let (cost, gained) = (5, 9);
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::GatheringDark);
    assert_eq!(STARTING_MANA - cost + gained, g.me().mana());
}

#[test]
fn coinery() {
    let (card_cost, taken) = (2, 3);
    let mut g = new_game(Side::Overlord, Args::default());
    let id = g.play_from_hand(CardName::Coinery);
    g.activate_ability(id, 1);
    assert_eq!(STARTING_MANA - card_cost + taken, g.me().mana());
    assert!(g.user.get_card(id).is_face_up());
    g.activate_ability(id, 1);
    assert_eq!(STARTING_MANA - card_cost + (taken * 2), g.me().mana());
}

#[test]
fn leyline() {
    let (card_cost, gained) = (2, 1);
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Leyline);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(STARTING_MANA - card_cost + gained, g.me().mana());
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(STARTING_MANA - card_cost + gained * 2, g.me().mana());
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(STARTING_MANA - card_cost + gained * 3, g.me().mana());
}

#[test]
fn ore_refinery() {
    let (card_cost, stored, taken) = (4, 12, 3);
    let mut g = new_game(Side::Overlord, Args::default());
    let id = g.play_from_hand(CardName::OreRefinery);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(STARTING_MANA - card_cost + taken, g.me().mana());
    assert_eq!((stored - taken).to_string(), g.user.get_card(id).arena_icon());
}

#[test]
fn crab() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Crab);
    set_up_minion_combat(&mut g);
    click_on_continue(&mut g);
    assert!(!g.user.data.raid_active());
}

#[test]
fn fire_goblin() {
    let (cost, gained) = (1, 1);
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::FireGoblin);
    set_up_minion_combat(&mut g);
    assert_eq!(STARTING_MANA - cost, g.me().mana());
    click_on_continue(&mut g);
    assert_eq!(STARTING_MANA - cost + gained, g.me().mana());
    assert_eq!(1, g.opponent.cards.discard_pile(PlayerName::User).len());
}

#[test]
fn toucan() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Toucan);
    set_up_minion_combat(&mut g);
    click_on_continue(&mut g);
    assert!(!g.user.data.raid_active());
}

#[test]
fn frog() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Frog);
    set_up_minion_combat(&mut g);
    click_on_continue(&mut g);
    assert!(!g.user.data.raid_active());
}

#[test]
fn scout() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Scout);
    set_up_minion_combat(&mut g);
    click_on_continue(&mut g);
    assert!(!g.user.data.raid_active());
    assert_eq!(1, g.opponent.this_player.actions());
}

#[test]
fn captain() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Captain);
    set_up_minion_combat(&mut g);
    click_on_continue(&mut g);
    assert!(!g.user.data.raid_active());
    assert_eq!(2, g.opponent.this_player.actions());
}
