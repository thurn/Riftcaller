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

use core_data::game_primitives::{RoomId, Side};
use game_data::card_name::CardName;
use protos::spelldawn::client_action::Action;
use protos::spelldawn::DrawCardAction;
use test_utils::test_game::{TestGame, TestSide};
use test_utils::*;

#[test]
pub fn incarnation_of_justice() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::IncarnationOfJustice);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    g.opponent_click(Button::EndRaid);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(g.client.cards.opponent_hand().len(), 0);
    g.perform(Action::DrawCard(DrawCardAction {}), g.opponent_id());
    assert_eq!(g.client.cards.opponent_hand().len(), 0);
}

#[test]
pub fn incarnation_of_justice_ancestral_knowledge() {
    let mut g = TestGame::new(TestSide::new(Side::Overlord)).build();
    g.create_and_play(CardName::IncarnationOfJustice);
    g.set_up_minion_combat();
    g.opponent_click(Button::NoWeapon);
    g.opponent_click(Button::EndRaid);
    g.create_and_play(CardName::AncestralKnowledge);
    assert_eq!(g.client.cards.opponent_hand().len(), 0);
}

#[test]
pub fn sentinel_sphinx() {
    let mut g = TestGame::new(
        TestSide::new(Side::Overlord)
            .face_up_defender(RoomId::Sanctum, CardName::TestMinionEndRaid),
    )
    .build();
    g.create_and_play_with_target(CardName::SentinelSphinx, RoomId::Vault);
    g.pass_turn(Side::Overlord);
    g.create_and_play(CardName::BackupPlan);

    g.initiate_raid(RoomId::Sanctum);
    // Normal minion: can activate backup plan
    assert!(g.opponent.cards.hand().find_ability_card(CardName::BackupPlan).can_play());
    g.opponent_click(Button::NoWeapon);

    g.initiate_raid(RoomId::Vault);
    g.click(Button::Summon);
    // Sentinel Sphinx: cannot activate backup plan
    assert!(!g.opponent.cards.hand().find_ability_card(CardName::BackupPlan).can_play());
}

#[test]
fn sentinel_sphinx_voidstep() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord).face_up_defender(RoomId::Vault, CardName::SentinelSphinx),
        )
        .build();
    g.create_and_play_with_target(CardName::Voidstep, RoomId::Vault);
    assert!(g.has(Button::NoWeapon));
}

#[test]
fn sentinel_sphinx_foebane() {
    let mut g = TestGame::new(TestSide::new(Side::Champion))
        .opponent(
            TestSide::new(Side::Overlord).face_up_defender(RoomId::Vault, CardName::SentinelSphinx),
        )
        .build();
    g.create_and_play_with_target(CardName::Foebane, RoomId::Vault);
    g.click(Button::ChooseOnPlay);
    g.initiate_raid(RoomId::Vault);
    assert!(!g.has(Button::Evade));
    g.click(Button::NoWeapon);
}
