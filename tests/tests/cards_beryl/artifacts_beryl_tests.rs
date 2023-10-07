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
use test_utils::test_helpers::WeaponStats;
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
        g.user.cards.artifacts().find_card(CardName::Pathfinder).attack_icon()
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
        g.user.cards.artifacts().find_card(CardName::Pathfinder).attack_icon()
    );
}

#[test]
fn staff_of_the_valiant() {
    let stats = WeaponStats { cost: 0, attack: 1, boost_cost: 2, boost: 1 };
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.pass_turn(Side::Champion);
    g.create_and_play(CardName::TestScheme3_10);
    g.create_and_play(CardName::TestInfernalMinion);
    g.create_and_play(CardName::TestInfernalMinion);
    g.pass_turn(Side::Overlord);
    g.create_and_play(CardName::StaffOfTheValiant);
    g.initiate_raid(test_constants::ROOM_ID);
    g.click_as_side(Button::Summon, Side::Overlord);
    g.click_on(g.user_id(), CardName::StaffOfTheValiant.displayed_name());
    let mana = test_constants::STARTING_MANA
        - test_helpers::cost_to_play_and_defeat(stats, test_constants::MINION_HEALTH);
    assert_eq!(g.me().mana(), mana,);
    assert_eq!(
        test_constants::MINION_HEALTH.to_string(),
        g.user.cards.artifacts().find_card(CardName::StaffOfTheValiant).attack_icon()
    );

    g.click_as_side(Button::Summon, Side::Overlord);
    g.click_on(g.user_id(), CardName::StaffOfTheValiant.displayed_name());
    assert_eq!(g.me().mana(), mana,);
    assert_eq!(
        test_constants::MINION_HEALTH.to_string(),
        g.user.cards.artifacts().find_card(CardName::StaffOfTheValiant).attack_icon()
    );

    g.click(Button::Score);
    assert_eq!(g.me().mana(), mana,);
    assert_eq!(
        test_constants::MINION_HEALTH.to_string(),
        g.user.cards.artifacts().find_card(CardName::StaffOfTheValiant).attack_icon()
    );
}

#[test]
fn triumph_return_to_hand() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Sanctum, CardName::TestAstralMinion)
                .face_up_defender(RoomId::Vault, CardName::TestAstralMinion),
        )
        .build();
    g.create_and_play(CardName::Triumph);
    g.initiate_raid(RoomId::Sanctum);
    g.click_weapon_name(CardName::Triumph);
    g.click(Button::EndRaid);

    assert_eq!(g.user.cards.room_defenders(RoomId::Sanctum).len(), 0);
    assert!(g.opponent.cards.hand().contains_card(CardName::TestAstralMinion));

    g.initiate_raid(RoomId::Vault);
    g.click_weapon_name(CardName::Triumph);
    g.click(Button::EndRaid);
    assert!(g.user.cards.room_defenders(RoomId::Vault).contains_card(CardName::TestAstralMinion));
}

#[test]
fn triumph_slow() {
    let stats = WeaponStats { cost: 8, attack: 0, boost_cost: 1, boost: 1 };
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord)
                .face_up_defender(RoomId::Sanctum, CardName::TestAstralMinion1Shield),
        )
        .build();
    g.create_and_play(CardName::Triumph);
    g.initiate_raid(RoomId::Sanctum);
    g.click_weapon_name(CardName::Triumph);
    assert_eq!(
        g.me().mana(),
        test_constants::STARTING_MANA
            - test_helpers::cost_to_play_and_defeat(stats, test_constants::MINION_HEALTH)
            - 2
    );
}
