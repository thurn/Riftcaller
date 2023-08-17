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
use protos::spelldawn::PlayerName;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn ennera_imris_blood_bound() {
    let gained = 1;
    let mut g = TestGame::new(TestSide::new(Side::Champion).sigil(CardName::RadiantSigil)).build();

    assert_eq!(0, g.user.cards.hand(PlayerName::User).len());
    g.spend_actions_until_turn_over(Side::Champion);
    g.spend_actions_until_turn_over(Side::Overlord);
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(STARTING_MANA + gained, g.me().mana());
}

#[test]
fn aris_fey_the_radiant_sun() {
    let mut g =
        TestGame::new(TestSide::new(Side::Champion).sigil(CardName::RestorationSigil)).build();

    g.spend_actions_until_turn_over(Side::Champion);
    g.create_and_play(CardName::TestMinionDealDamage);
    g.set_up_minion_combat();
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(0, g.user.cards.discard_pile(PlayerName::User).len());
    g.click(Buttons::NoWeapon);
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(1, g.user.cards.discard_pile(PlayerName::User).len());
}

#[test]
fn telantes_dugoth_earthbreaker() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).sigil(CardName::ForgeSigil)).build();

    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(0, g.user.cards.discard_pile(PlayerName::Opponent).len());
    g.click(Buttons::EndRaid);
    assert_eq!(1, g.user.cards.discard_pile(PlayerName::Opponent).len());
}

#[test]
fn andvari_est_nights_warden() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).sigil(CardName::CrabSigil))
        .opponent(
            TestSide::new(Side::Overlord)
                .deck_top(CardName::TestChampionSpell)
                .deck_top(CardName::TestChampionSpell)
                .deck_top(CardName::TestScheme3_15)
                .deck_top(CardName::TestChampionSpell)
                .deck_top(CardName::TestChampionSpell),
        )
        .build();

    g.initiate_raid(RoomId::Vault);
    g.click(Buttons::Score);
    assert_eq!(15, g.me().score())
}

#[test]
fn ubras_efaris_time_shaper() {
    let mut g = TestGame::new(TestSide::new(Side::Champion).sigil(CardName::ArcaneSigil)).build();

    assert_eq!(3, g.me().actions());
    g.create_and_play(CardName::TestChampionSpell);
    assert_eq!(2, g.me().actions());
    g.create_and_play(CardName::TestChampionSpell);
    assert_eq!(2, g.me().actions());
    g.create_and_play(CardName::TestChampionSpell);
    assert_eq!(1, g.me().actions());
}
