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

use core_data::game_primitives::Side;
use game_data::card_definition::Resonance;
use game_data::card_name::CardName;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::test_helpers::WeaponStats;
use test_utils::*;

#[test]
fn arcane_recovery() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).mana(5)).build();
    g.create_and_play(CardName::ArcaneRecovery);
    assert_eq!(9, g.me().mana());
    assert_eq!(9, g.opponent.other_player.mana())
}

#[test]
fn eldritch_surge() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).mana(0)).build();
    g.create_and_play(CardName::EldritchSurge);
    assert_eq!(3, g.me().mana());
    assert_eq!(3, g.opponent.other_player.mana())
}

#[test]
fn lodestone() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let id = g.create_and_play(CardName::Lodestone);
    assert_eq!("12", g.client.get_card(id).arena_icon());
    g.activate_ability(id, 1);
    assert_eq!(test_constants::STARTING_MANA - 1 + 2, g.me().mana());
    assert_eq!(2, g.me().actions());
    assert_eq!("10", g.client.get_card(id).arena_icon());
}

#[test]
fn mana_battery() {
    let card_cost = 0;
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let id = g.create_and_play(CardName::ManaBattery);
    g.activate_ability(id, 1);
    g.pass_turn(Side::Champion);
    g.pass_turn(Side::Overlord);
    assert!(g.dawn());
    assert_eq!(test_constants::STARTING_MANA - card_cost + 1, g.me().mana());
    g.pass_turn(Side::Champion);
    g.pass_turn(Side::Overlord);
    assert_eq!(test_constants::STARTING_MANA - card_cost + 2, g.me().mana());
    g.pass_turn(Side::Champion);
    g.pass_turn(Side::Overlord);
    assert_eq!(test_constants::STARTING_MANA - card_cost + 3, g.me().mana());
    g.pass_turn(Side::Champion);
    g.pass_turn(Side::Overlord);
    assert_eq!(test_constants::STARTING_MANA - card_cost + 3, g.me().mana());
}

#[test]
fn contemplate() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    assert_eq!(0, g.client.cards.hand().len());
    g.create_and_play(CardName::Contemplate);
    assert_eq!(1, g.client.cards.hand().len());
    assert_eq!(test_constants::STARTING_MANA + 2, g.me().mana());
}

#[test]
fn ancestral_knowledge() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    assert_eq!(0, g.client.cards.hand().len());
    g.create_and_play(CardName::AncestralKnowledge);
    assert_eq!(3, g.client.cards.hand().len());
    assert_eq!(test_constants::STARTING_MANA - 1, g.me().mana());
}

#[test]
fn simple_blade() {
    let stats = WeaponStats { cost: 4, attack: 2, boost_cost: 1, boost: 1 };
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::SimpleBlade);
    g.fire_weapon_combat_abilities(Resonance::mortal(), CardName::SimpleBlade);
    assert_eq!(
        test_constants::STARTING_MANA
            - test_helpers::cost_to_play_and_defeat(stats, test_constants::MINION_HEALTH),
        g.me().mana()
    );
}

#[test]
fn simple_axe() {
    let stats = WeaponStats { cost: 4, attack: 3, boost_cost: 3, boost: 1 };
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::SimpleAxe);
    g.fire_weapon_combat_abilities(Resonance::mortal(), CardName::SimpleAxe);
    assert_eq!(
        test_constants::STARTING_MANA
            - test_helpers::cost_to_play_and_defeat(stats, test_constants::MINION_HEALTH),
        g.me().mana()
    );
}

#[test]
fn simple_bow() {
    let stats = WeaponStats { cost: 0, attack: 1, boost_cost: 2, boost: 1 };
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::SimpleBow);
    g.fire_weapon_combat_abilities(Resonance::astral(), CardName::SimpleBow);
    assert_eq!(
        test_constants::STARTING_MANA
            - test_helpers::cost_to_play_and_defeat(stats, test_constants::MINION_HEALTH),
        g.me().mana()
    );
}

#[test]
fn simple_club() {
    let stats = WeaponStats { cost: 2, attack: 2, boost_cost: 1, boost: 1 };
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::SimpleClub);
    g.fire_weapon_combat_abilities(Resonance::astral(), CardName::SimpleClub);
    assert_eq!(
        test_constants::STARTING_MANA
            - test_helpers::cost_to_play_and_defeat(stats, test_constants::MINION_HEALTH),
        g.me().mana()
    );
}

#[test]
fn simple_hammer() {
    let stats = WeaponStats { cost: 3, attack: 1, boost_cost: 1, boost: 1 };
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::SimpleHammer);
    g.fire_weapon_combat_abilities(Resonance::infernal(), CardName::SimpleHammer);
    assert_eq!(
        test_constants::STARTING_MANA
            - test_helpers::cost_to_play_and_defeat(stats, test_constants::MINION_HEALTH),
        g.me().mana()
    );
}

#[test]
fn simple_spear() {
    let stats = WeaponStats { cost: 4, attack: 0, boost_cost: 3, boost: 5 };
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::SimpleSpear);
    g.fire_weapon_combat_abilities(Resonance::infernal(), CardName::SimpleSpear);
    assert_eq!(
        test_constants::STARTING_MANA
            - test_helpers::cost_to_play_and_defeat(stats, test_constants::MINION_HEALTH),
        g.me().mana()
    );
}

#[test]
fn ethereal_blade() {
    let (card_cost, activation_cost) = (1, 1);
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::EtherealBlade);
    g.fire_weapon_combat_abilities(Resonance::mortal(), CardName::EtherealBlade);
    assert_eq!(test_constants::STARTING_MANA - card_cost - (4 * activation_cost), g.me().mana());
    g.click(Button::Score);
    assert_eq!(0, g.client.cards.discard_pile().len());
    assert_eq!(1, g.client.cards.artifacts().len());
    g.click(Button::EndRaid);
    assert_eq!(1, g.client.cards.discard_pile().len());
    assert_eq!(0, g.client.cards.artifacts().len());
}
