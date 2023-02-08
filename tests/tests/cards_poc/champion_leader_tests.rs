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
use test_utils::*;

#[test]
fn ennera_imris_blood_bound() {
    let (cost, gained) = (0, 1);
    let mut g = new_game(Side::Champion, Args::default());
    g.play_from_hand(CardName::EnneraImrisBloodBound);
    assert_eq!(0, g.user.cards.hand(PlayerName::User).len());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(STARTING_MANA + gained - cost, g.me().mana());
}
