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
fn ubras_efaris() {
    let mut g =
        TestGame::new(TestSide::new(Side::Overlord).riftcaller(CardName::UbrasEfarisTimeShaper))
            .build();

    assert_eq!(3, g.me().actions());
    g.create_and_play(CardName::TestRitual);
    assert_eq!(2, g.me().actions());
    g.create_and_play(CardName::TestRitual);
    assert_eq!(2, g.me().actions());
    g.create_and_play(CardName::TestRitual);
    assert_eq!(1, g.me().actions());
}
