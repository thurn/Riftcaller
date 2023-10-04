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
use game_data::primitives::{RoomId, Side};
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn pathfinder() {
    let (base_attack, bonus) = (1, 2);
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::Pathfinder);
    g.setup_raid_target(CardName::TestInfernalMinion);
    g.initiate_raid(test_constants::ROOM_ID);
    assert_eq!(
        (base_attack + bonus).to_string(),
        g.user.cards.artifacts().find_card(CardName::Pathfinder).bottom_right_icon()
    );
}

#[test]
fn pathfinder_inner_room() {
    let base_attack = 1;
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::Pathfinder);
    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(
        base_attack.to_string(),
        g.user.cards.artifacts().find_card(CardName::Pathfinder).bottom_right_icon()
    );
}
