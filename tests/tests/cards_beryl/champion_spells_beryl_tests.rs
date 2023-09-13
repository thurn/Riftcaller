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
use protos::spelldawn::PlayerName;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::TestSessionHelpers;

#[test]
fn restoration() {
    let cost = 1;
    let mut g = TestGame::new(
        TestSide::new(Side::Champion).in_discard_face_up(CardName::TestWeaponAbyssal),
    )
    .build();
    assert!(g.user.cards.left_items().is_empty());
    g.create_and_play(CardName::Restoration);
    assert_eq!(g.user.cards.hand(PlayerName::User), vec!["Test Weapon Abyssal"]);
    let id = g.user.cards.cards_in_hand(PlayerName::User).next().unwrap().id();
    g.play_card(id, g.user_id(), None);
    assert!(g.user.cards.hand(PlayerName::User).is_empty());
    assert_eq!(g.user.cards.left_items(), vec!["Test Weapon Abyssal"]);
    assert_eq!(g.me().mana(), test_constants::STARTING_MANA - cost - test_constants::WEAPON_COST);
}
