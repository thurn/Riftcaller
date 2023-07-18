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

use cards_test::test_cards::MINION_HEALTH;
use game_data::card_name::CardName;
use game_data::primitives::{Lineage, Side};
use protos::spelldawn::PlayerName;
use test_utils::*;

#[test]
fn arcane_recovery() {
    let mut g = new_game(Side::Champion, Args { mana: 5, ..Args::default() });
    g.create_and_play(CardName::ArcaneRecovery);
    assert_eq!(9, g.me().mana());
    assert_eq!(9, g.opponent.other_player.mana())
}

#[test]
fn eldritch_surge() {
    let mut g = new_game(Side::Champion, Args { mana: 0, ..Args::default() });
    g.create_and_play(CardName::EldritchSurge);
    assert_eq!(3, g.me().mana());
    assert_eq!(3, g.opponent.other_player.mana())
}

#[test]
fn lodestone() {
    let mut g = new_game(Side::Champion, Args::default());
    let id = g.create_and_play(CardName::Lodestone);
    assert_eq!("12", g.user.get_card(id).arena_icon());
    g.activate_ability(id, 1);
    assert_eq!(STARTING_MANA - 1 + 2, g.me().mana());
    assert_eq!(1, g.me().actions());
    assert_eq!("10", g.user.get_card(id).arena_icon());
}

#[test]
fn mana_battery() {
    let card_cost = 0;
    let mut g = new_game(Side::Champion, Args::default());
    let id = g.create_and_play(CardName::ManaBattery);
    g.activate_ability(id, 1);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    assert!(g.dawn());
    assert_eq!(STARTING_MANA - card_cost + 1, g.me().mana());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    assert_eq!(STARTING_MANA - card_cost + 2, g.me().mana());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    assert_eq!(STARTING_MANA - card_cost + 3, g.me().mana());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    assert_eq!(STARTING_MANA - card_cost + 3, g.me().mana());
}

#[test]
fn contemplate() {
    let mut g = new_game(Side::Champion, Args::default());
    assert_eq!(0, g.user.cards.hand(PlayerName::User).len());
    g.create_and_play(CardName::Contemplate);
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(STARTING_MANA + 2, g.me().mana());
}

#[test]
fn ancestral_knowledge() {
    let mut g = new_game(Side::Champion, Args::default());
    assert_eq!(0, g.user.cards.hand(PlayerName::User).len());
    g.create_and_play(CardName::AncestralKnowledge);
    assert_eq!(3, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(STARTING_MANA - 1, g.me().mana());
}

#[test]
fn simple_blade() {
    let stats = WeaponStats { cost: 4, attack: 2, boost_cost: 1, boost: 1 };
    let mut g = new_game(Side::Champion, Args::default());
    g.create_and_play(CardName::SimpleBlade);
    fire_weapon_combat_abilities(&mut g, Lineage::Mortal, CardName::SimpleBlade);
    assert_eq!(STARTING_MANA - cost_to_play_and_defeat(stats, MINION_HEALTH), g.me().mana());
}

#[test]
fn simple_axe() {
    let stats = WeaponStats { cost: 4, attack: 3, boost_cost: 3, boost: 1 };
    let mut g = new_game(Side::Champion, Args::default());
    g.create_and_play(CardName::SimpleAxe);
    fire_weapon_combat_abilities(&mut g, Lineage::Mortal, CardName::SimpleAxe);
    assert_eq!(STARTING_MANA - cost_to_play_and_defeat(stats, MINION_HEALTH), g.me().mana());
}

#[test]
fn simple_bow() {
    let stats = WeaponStats { cost: 0, attack: 1, boost_cost: 2, boost: 1 };
    let mut g = new_game(Side::Champion, Args::default());
    g.create_and_play(CardName::SimpleBow);
    fire_weapon_combat_abilities(&mut g, Lineage::Abyssal, CardName::SimpleBow);
    assert_eq!(STARTING_MANA - cost_to_play_and_defeat(stats, MINION_HEALTH), g.me().mana());
}

#[test]
fn simple_club() {
    let stats = WeaponStats { cost: 2, attack: 2, boost_cost: 1, boost: 1 };
    let mut g = new_game(Side::Champion, Args::default());
    g.create_and_play(CardName::SimpleClub);
    fire_weapon_combat_abilities(&mut g, Lineage::Abyssal, CardName::SimpleClub);
    assert_eq!(STARTING_MANA - cost_to_play_and_defeat(stats, MINION_HEALTH), g.me().mana());
}

#[test]
fn simple_hammer() {
    let stats = WeaponStats { cost: 3, attack: 1, boost_cost: 1, boost: 1 };
    let mut g = new_game(Side::Champion, Args::default());
    g.create_and_play(CardName::SimpleHammer);
    fire_weapon_combat_abilities(&mut g, Lineage::Infernal, CardName::SimpleHammer);
    assert_eq!(STARTING_MANA - cost_to_play_and_defeat(stats, MINION_HEALTH), g.me().mana());
}

#[test]
fn simple_spear() {
    let stats = WeaponStats { cost: 4, attack: 0, boost_cost: 3, boost: 5 };
    let mut g = new_game(Side::Champion, Args::default());
    g.create_and_play(CardName::SimpleSpear);
    fire_weapon_combat_abilities(&mut g, Lineage::Infernal, CardName::SimpleSpear);
    assert_eq!(STARTING_MANA - cost_to_play_and_defeat(stats, MINION_HEALTH), g.me().mana());
}

#[test]
fn ethereal_blade() {
    let (card_cost, activation_cost) = (1, 1);
    let mut g = new_game(Side::Champion, Args::default());
    g.create_and_play(CardName::EtherealBlade);
    fire_weapon_combat_abilities(&mut g, Lineage::Mortal, CardName::EtherealBlade);
    assert_eq!(STARTING_MANA - card_cost - (4 * activation_cost), g.me().mana());
    click_on_score(&mut g);
    assert_eq!(0, g.user.cards.discard_pile(PlayerName::User).len());
    assert_eq!(1, g.user.cards.left_items().len());
    click_on_end_raid(&mut g);
    assert_eq!(1, g.user.cards.discard_pile(PlayerName::User).len());
    assert_eq!(0, g.user.cards.left_items().len());
}
