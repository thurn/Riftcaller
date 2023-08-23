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
use game_data::primitives::{Resonance, RoomId, Side};
use test_utils::client_interface::HasText;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn test_attack_weapon() {
    let card_cost = 3;
    let ability_cost = 1;
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::TestAttackWeapon);
    g.fire_weapon_combat_abilities(Resonance::Infernal, CardName::TestAttackWeapon);
    assert_eq!(test_constants::STARTING_MANA - card_cost - ability_cost, g.me().mana());
    assert!(g.user.data.raid_active());
    assert!(g.user.interface.controls().has_text("End Raid"));
}

#[test]
fn marauders_axe() {
    let card_cost = 5;
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let id = g.add_to_hand(CardName::MaraudersAxe);
    assert_eq!(card_cost.to_string(), g.user.cards.get(id).top_left_icon());
    g.initiate_raid(RoomId::Crypts);
    g.click(Button::EndRaid);
    assert_eq!((card_cost - 2).to_string(), g.user.cards.get(id).top_left_icon());
    g.play_card(id, g.user_id(), None);
    assert_eq!(test_constants::STARTING_MANA - card_cost + 2, g.me().mana());
}

#[test]
fn keen_halberd() {
    let (card_cost, activation_cost) = (3, 2);
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::KeenHalberd);
    g.setup_raid_target(CardName::TestMinionShield2Abyssal);
    g.initiate_raid(test_constants::ROOM_ID);
    g.opponent_click(Button::Summon);
    g.click_on(g.user_id(), "Keen Halberd");
    assert_eq!(
        test_constants::STARTING_MANA - card_cost - (2 * activation_cost) - 1, /* remaining shield */
        g.me().mana()
    );
}

#[test]
fn bow_of_the_alliance() {
    let (card_cost, activation_cost) = (3, 1);
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::BowOfTheAlliance);
    g.create_and_play(CardName::BowOfTheAlliance);
    g.fire_weapon_combat_abilities(Resonance::Mortal, CardName::BowOfTheAlliance);
    assert_eq!(
        test_constants::STARTING_MANA - (2 * card_cost) - (2 * activation_cost),
        g.me().mana()
    );
}
