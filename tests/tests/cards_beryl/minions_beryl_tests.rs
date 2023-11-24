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
use game_data::card_name::CardName;
use protos::spelldawn::client_action::Action;
use protos::spelldawn::DrawCardAction;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
pub fn incarnation_of_justice() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::IncarnationOfJustice);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    g.opponent_click(Button::EndRaid);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(g.user.cards.opponent_hand().len(), 0);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(g.user.cards.opponent_hand().len(), 0);
}

#[test]
pub fn incarnation_of_justice_ancestral_knowledge() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::IncarnationOfJustice);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    g.opponent_click(Button::EndRaid);
    g.create_and_play(CardName::AncestralKnowledge);
    assert_eq!(g.user.cards.opponent_hand().len(), 0);
}
