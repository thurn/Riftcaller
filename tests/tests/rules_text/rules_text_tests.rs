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

use core_ui::icons;
use game_data::card_name::CardName;
use game_data::primitives::Side;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn test_keyword_aggregation() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    let id = g.create_and_play(CardName::TestMinionDealDamageEndRaid);
    assert_eq!(
        format!("{}Combat: Deal 1 damage. End the raid.", icons::TRIGGER),
        g.user.cards.get(id).rules_text()
    );
}
