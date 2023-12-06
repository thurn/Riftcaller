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

use core_data::game_primitives::Side;
use game_data::card_name::CardName;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn test_card_stored_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::TestProjectTriggeredAbilityTakeManaAtDusk);
    g.pass_turn(Side::Covenant);
    assert!(g.dawn());
    assert_eq!(test_constants::STARTING_MANA, g.me().mana());
    g.move_to_end_step(Side::Riftcaller);
    g.summon_project(id);
    assert_eq!(test_constants::STARTING_MANA - test_constants::SUMMON_PROJECT_COST, g.me().mana());
    g.click(Button::StartTurn);
    assert!(g.dusk());
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::SUMMON_PROJECT_COST
            + test_constants::MANA_TAKEN,
        g.me().mana()
    );
    assert_eq!(
        (test_constants::MANA_STORED - test_constants::MANA_TAKEN).to_string(),
        g.client.get_card(id).arena_icon()
    );
}

#[test]
fn gemcarver() {
    let (card_cost, taken) = (2, 3);
    let mut g = TestGame::new(TestSide::new(Side::Covenant)).build();
    let id = g.create_and_play(CardName::Gemcarver);
    g.pass_turn(Side::Covenant);
    g.move_to_end_step(Side::Riftcaller);
    g.summon_project(id);
    g.click(Button::StartTurn);
    assert_eq!(test_constants::STARTING_MANA - card_cost + taken, g.me().mana());
    g.pass_turn(Side::Covenant);
    g.pass_turn(Side::Riftcaller);
    assert_eq!(test_constants::STARTING_MANA - card_cost + taken * 2, g.me().mana());
    assert_eq!(2, g.client.cards.hand().len());
    g.pass_turn(Side::Covenant);
    g.pass_turn(Side::Riftcaller);
    assert_eq!(test_constants::STARTING_MANA - card_cost + taken * 3, g.me().mana());
    assert_eq!(4, g.client.cards.hand().len());
}

#[test]
fn spike_trap() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant))
        .opponent(TestSide::new(Side::Riftcaller).hand_size(5))
        .build();
    g.create_and_play(CardName::SpikeTrap);
    g.progress_room_times(2);
    g.pass_turn(Side::Covenant);

    assert!(g.dawn());
    assert_eq!(5, g.client.cards.opponent_hand().len());
    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(1, g.client.cards.opponent_hand().len());
}

#[test]
fn spike_trap_no_counters() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant))
        .opponent(TestSide::new(Side::Riftcaller).hand_size(5))
        .build();
    g.create_and_play(CardName::SpikeTrap);
    g.pass_turn(Side::Covenant);
    assert!(g.dawn());
    assert_eq!(5, g.client.cards.opponent_hand().len());
    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(3, g.client.cards.opponent_hand().len());
}

#[test]
fn spike_trap_victory() {
    let mut g = TestGame::new(TestSide::new(Side::Covenant))
        .opponent(TestSide::new(Side::Riftcaller).hand_size(0))
        .build();
    g.create_and_play(CardName::SpikeTrap);
    g.progress_room_times(2);
    g.pass_turn(Side::Covenant);

    assert!(g.dawn());
    g.initiate_raid(test_constants::ROOM_ID);
    assert!(g.is_victory_for_player(Side::Covenant));
}
