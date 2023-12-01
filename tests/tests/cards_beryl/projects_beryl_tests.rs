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
use protos::riftcaller::client_action::Action;
use protos::riftcaller::DrawCardAction;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::TestSessionHelpers;

#[test]
pub fn magistrates_thronehall() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::MagistratesThronehall);
    g.summon_project(id);
    g.pass_turn(Side::Overlord);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(g.client.cards.opponent_hand().len(), 2);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(g.client.cards.opponent_hand().len(), 2);
}

#[test]
pub fn magistrates_thronehall_ancestral_knowledge() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::MagistratesThronehall);
    g.summon_project(id);
    g.pass_turn(Side::Overlord);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    g.create_and_play(CardName::AncestralKnowledge);
    assert_eq!(g.client.cards.opponent_hand().len(), 2);
}
