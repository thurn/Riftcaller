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

use core_data::game_primitives::{RoomId, Side};
use game_data::card_name::CardName;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn ennera_imris_blood_bound() {
    let gained = 1;
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::EnneraImrisBloodBound))
            .build();

    assert_eq!(0, g.client.cards.hand().len());
    g.pass_turn(Side::Riftcaller);
    g.pass_turn(Side::Covenant);
    assert_eq!(0, g.client.cards.hand().len());
    assert_eq!(test_constants::STARTING_MANA + gained, g.me().mana());
}

#[test]
fn aris_fey_the_radiant_sun() {
    let mut g = TestGame::new(
        TestSide::new(Side::Riftcaller).hand_size(1).identity(CardName::ArisFeyTheRadiantSun),
    )
    .build();

    g.pass_turn(Side::Riftcaller);
    g.create_and_play(CardName::TestMinionDealDamage);
    g.set_up_minion_combat();
    assert_eq!(1, g.client.cards.hand().len());
    assert_eq!(0, g.client.cards.discard_pile().len());
    g.click(Button::NoWeapon);
    assert_eq!(1, g.client.cards.hand().len());
    assert_eq!(1, g.client.cards.discard_pile().len());
}

#[test]
fn telantes_dugoth_earthbreaker() {
    let mut g = TestGame::new(
        TestSide::new(Side::Riftcaller).identity(CardName::TelantesDugothEarthbreaker),
    )
    .build();

    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(0, g.client.cards.opponent_discard_pile().len());
    g.click(Button::EndRaid);
    assert_eq!(1, g.client.cards.opponent_discard_pile().len());
}

#[test]
fn andvari_est_nights_warden() {
    let mut g =
        TestGame::new(TestSide::new(Side::Riftcaller).identity(CardName::AndvariEstNightsWarden))
            .opponent(
                TestSide::new(Side::Covenant)
                    .deck_top(CardName::TestSpell)
                    .deck_top(CardName::TestSpell)
                    .deck_top(CardName::TestScheme3_10)
                    .deck_top(CardName::TestSpell)
                    .deck_top(CardName::TestSpell),
            )
            .build();

    g.initiate_raid(RoomId::Vault);
    g.click(Button::Score);
    assert_eq!(10, g.me().score())
}
