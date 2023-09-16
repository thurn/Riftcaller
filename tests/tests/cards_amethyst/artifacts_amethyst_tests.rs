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
use protos::spelldawn::client_action::Action;
use protos::spelldawn::{DrawCardAction, RoomIdentifier};
use test_utils::client_interface::HasText;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
fn invisibility_ring() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.add_to_hand(CardName::TestScheme3_10);
    g.add_to_hand(CardName::TestScheme3_10);

    g.create_and_play(CardName::InvisibilityRing);
    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(2, g.user.interface.card_anchor_nodes().len());
    assert_eq!(vec!["Score!"], g.user.interface.card_anchor_nodes()[0].get_text());
    assert_eq!(vec!["Score!"], g.user.interface.card_anchor_nodes()[1].get_text());
    g.click(Button::EndRaid);
    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(1, g.user.interface.card_anchor_nodes().len());
    assert_eq!(vec!["Score!"], g.user.interface.card_anchor_nodes()[0].get_text());
}

#[test]
fn accumulator() {
    let card_cost = 3;
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let id = g.create_and_play(CardName::Accumulator);
    g.initiate_raid(RoomId::Crypts);
    g.click(Button::EndRaid);
    assert_eq!("1", g.user.get_card(id).arena_icon());
    g.activate_ability(id, 1);
    assert_eq!(test_constants::STARTING_MANA + 2 - card_cost, g.me().mana())
}

#[test]
fn mage_gloves() {
    let card_cost = 5;
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let id = g.create_and_play(CardName::MageGloves);
    assert_eq!("12", g.user.get_card(id).arena_icon());
    assert_eq!(
        vec![RoomIdentifier::Vault, RoomIdentifier::Sanctum, RoomIdentifier::Crypts],
        g.user.cards.get(test_helpers::ability_id(id, 1)).valid_rooms()
    );
    g.activate_ability_with_target(id, 1, RoomId::Crypts);
    g.click(Button::EndRaid);
    assert_eq!(test_constants::STARTING_MANA + 3 - card_cost, g.me().mana());
    assert_eq!("9", g.user.get_card(id).arena_icon());
    assert_eq!(
        vec![RoomIdentifier::Vault, RoomIdentifier::Sanctum],
        g.user.cards.get(test_helpers::ability_id(id, 1)).valid_rooms()
    );
}

#[test]
fn mage_gloves_play_after_raid() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let id = g.add_to_hand(CardName::MageGloves);
    g.initiate_raid(RoomId::Sanctum);
    g.click(Button::EndRaid);
    g.play_card(id, g.user_id(), None);
    assert_eq!("12", g.user.get_card(id).arena_icon());
    assert_eq!(
        vec![RoomIdentifier::Vault, RoomIdentifier::Crypts],
        g.user.cards.get(test_helpers::ability_id(id, 1)).valid_rooms()
    );
}

#[test]
#[should_panic]
fn mage_gloves_repeat_panic() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let id = g.create_and_play(CardName::MageGloves);
    g.activate_ability_with_target(id, 1, RoomId::Crypts);
    g.click(Button::EndRaid);
    g.activate_ability_with_target(id, 1, RoomId::Crypts);
}

#[test]
fn magical_resonator() {
    let card_cost = 1;
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let id = g.create_and_play(CardName::MagicalResonator);
    g.activate_ability(id, 1);
    assert_eq!(test_constants::STARTING_MANA - card_cost + 3, g.me().mana());
    assert_eq!("6", g.user.get_card(id).arena_icon());
}

#[test]
#[should_panic]
fn magical_resonator_cannot_activate_twice() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    let id = g.create_and_play(CardName::MagicalResonator);
    g.activate_ability(id, 1);
    g.activate_ability(id, 1);
}

#[test]
fn dark_grimoire() {
    let mut g = TestGame::new(TestSide::new(Side::Champion)).build();
    g.create_and_play(CardName::DarkGrimoire);
    assert_eq!(0, g.user.cards.hand().len());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    assert_eq!(2, g.user.cards.hand().len());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    assert_eq!(3, g.user.cards.hand().len());
}
