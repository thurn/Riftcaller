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
use game_data::primitives::Side;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn test_card_stored_mana() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestProjectTriggeredAbilityTakeManaAtDusk);
    g.pass_turn(Side::Overlord);
    assert!(g.dawn());
    assert_eq!(test_constants::STARTING_MANA, g.me().mana());
    g.move_to_end_step(Side::Champion);
    g.unveil_card(id);
    assert_eq!(test_constants::STARTING_MANA - test_constants::UNVEIL_COST, g.me().mana());
    g.click(Button::StartTurn);
    assert!(g.dusk());
    assert_eq!(
        test_constants::STARTING_MANA - test_constants::UNVEIL_COST + test_constants::MANA_TAKEN,
        g.me().mana()
    );
    assert_eq!(
        (test_constants::MANA_STORED - test_constants::MANA_TAKEN).to_string(),
        g.user.get_card(id).arena_icon()
    );
}

#[test]
fn gemcarver() {
    let (card_cost, taken) = (2, 3);
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::Gemcarver);
    g.pass_turn(Side::Overlord);
    g.move_to_end_step(Side::Champion);
    g.unveil_card(id);
    g.click(Button::StartTurn);
    assert_eq!(test_constants::STARTING_MANA - card_cost + taken, g.me().mana());
    g.pass_turn(Side::Overlord);
    g.pass_turn(Side::Champion);
    assert_eq!(test_constants::STARTING_MANA - card_cost + taken * 2, g.me().mana());
    assert_eq!(2, g.user.cards.hand().len());
    g.pass_turn(Side::Overlord);
    g.pass_turn(Side::Champion);
    assert_eq!(test_constants::STARTING_MANA - card_cost + taken * 3, g.me().mana());
    assert_eq!(4, g.user.cards.hand().len());
}

#[test]
fn spike_trap() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(5))
        .build();
    g.create_and_play(CardName::SpikeTrap);
    g.progress_room_times(2);
    g.pass_turn(Side::Overlord);

    assert!(g.dawn());
    assert_eq!(5, g.user.cards.opponent_hand().len());
    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(1, g.user.cards.opponent_hand().len());
}

#[test]
fn spike_trap_no_counters() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(5))
        .build();
    g.create_and_play(CardName::SpikeTrap);
    g.pass_turn(Side::Overlord);
    assert!(g.dawn());
    assert_eq!(5, g.user.cards.opponent_hand().len());
    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(3, g.user.cards.opponent_hand().len());
}

#[test]
fn spike_trap_victory() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord))
        .opponent(TestSide::new(Side::Champion).hand_size(0))
        .build();
    g.create_and_play(CardName::SpikeTrap);
    g.progress_room_times(2);
    g.pass_turn(Side::Overlord);

    assert!(g.dawn());
    g.initiate_raid(test_constants::ROOM_ID);
    assert!(g.is_victory_for_player(Side::Overlord));
}
