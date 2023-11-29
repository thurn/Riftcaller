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

use core_data::game_primitives::{RoomId, Side};
use core_ui::icons;
use game_data::card_name::CardName;
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
    assert_eq!(1, g.user.cards.opponent_discard_pile().len());
    assert_eq!(4, g.user.cards.opponent_hand().len());
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
fn shadow_lurker_outer_room() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.add_to_hand(CardName::ShadowLurker);
    assert_eq!("2", g.user.get_card(id).bottom_right_icon());
    let id = g.create_and_play(CardName::ShadowLurker);
    assert_eq!("4", g.user.get_card(id).bottom_right_icon());
    g.set_up_minion_combat_with_action(|g| {
        g.create_and_play(CardName::TestAstralWeapon);
    });
    g.click_on(g.opponent_id(), "Test Astral Weapon");
    assert_eq!(test_constants::STARTING_MANA - 5, g.opponent.this_player.mana());
}

#[test]
fn shadow_lurker_inner_room() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play_with_target(CardName::ShadowLurker, RoomId::Sanctum);
    assert_eq!("2", g.user.get_card(id).bottom_right_icon());
}

#[test]
fn sphinx_of_winters_breath_discard_even() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).deck_top(CardName::Test0CostSpell))
        .build();

    g.create_and_play(CardName::SphinxOfWintersBreath);
    g.set_up_minion_combat_with_action(|g| {
        g.add_to_hand(CardName::Test0CostSpell);
    });
    g.opponent_click(Button::NoWeapon);
    assert_eq!(vec!["Test 0 Cost Spell"], g.opponent.cards.discard_pile().names());
    assert!(g.user.data.raid_active());
}

#[test]
fn sphinx_of_winters_breath_discard_odd() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).deck_top(CardName::Test1CostSpell))
        .build();

    g.create_and_play(CardName::SphinxOfWintersBreath);
    g.set_up_minion_combat_with_action(|g| {
        g.add_to_hand(CardName::Test1CostSpell);
    });
    g.opponent_click(Button::NoWeapon);
    assert_eq!(vec!["Test 1 Cost Spell"], g.opponent.cards.discard_pile().names());
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
    assert_eq!(2, g.opponent.cards.discard_pile().len());
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
    assert_eq!(4, g.opponent.cards.discard_pile().len());
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
