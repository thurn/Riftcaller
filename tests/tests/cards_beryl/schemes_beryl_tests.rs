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
fn ethereal_form() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::EtherealForm);
    g.pass_turn(Side::Overlord);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::Score);
    g.opponent_click(Button::EndRaid);
    g.pass_turn(Side::Champion);
    assert_eq!(g.user.other_player.score(), 10);
    assert!(g.user.cards.opponent_score_area().contains_card(CardName::EtherealForm));
    g.activate_ability(id, 0);
    assert_eq!(g.user.other_player.score(), 0);
    assert_eq!(g.user.cards.opponent_score_area().len(), 0);
}

#[test]
fn ethereal_form_cannot_activate_in_play() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::EtherealForm);
    assert!(g.activate_ability_with_result(id, 0).is_err());
}
