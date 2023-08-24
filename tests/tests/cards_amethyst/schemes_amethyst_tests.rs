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
use protos::spelldawn::client_action::Action;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{DrawCardAction, ObjectPositionCharacter, PlayerName};
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn gold_mine() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::GoldMine);
    g.level_up_room_times(4);
    assert_eq!(g.me().score(), 20);
    assert_eq!(
        test_constants::STARTING_MANA - 4 /* level cost */ + 7, /* gained */
        g.me().mana()
    );
    assert_eq!(
        g.user.get_card(id).position(),
        Position::Character(ObjectPositionCharacter { owner: PlayerName::User.into() })
    );
}

#[test]
fn activate_reinforcements() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::ActivateReinforcements);
    let minion = g.create_and_play(CardName::TestMinionEndRaid);
    assert!(!g.user.get_card(minion).is_face_up());
    g.level_up_room_times(5);
    assert_eq!(g.me().score(), 30);
    assert!(g.user.get_card(minion).is_face_up());
    assert_eq!(test_constants::STARTING_MANA - 5, g.me().mana());
    assert_eq!(
        g.user.get_card(id).position(),
        Position::Character(ObjectPositionCharacter { owner: PlayerName::User.into() })
    );
}

#[test]
fn research_project() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::ResearchProject);
    g.level_up_room_times(2);
    g.pass_turn(Side::Overlord);
    g.pass_turn(Side::Champion);
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    g.level_up_room_times(1);
    assert_eq!(3, g.user.cards.hand(PlayerName::User).len());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.pass_turn(Side::Overlord);
    g.pass_turn(Side::Champion);
    assert_eq!(6, g.user.cards.hand(PlayerName::User).len());
}
