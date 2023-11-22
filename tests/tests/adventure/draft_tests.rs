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

use adventure_data::adventure::{CardChoice, DraftData, TileEntity};
use core_data::adventure_primitives::Coins;
use core_data::game_primitives::Side;
use game_data::card_name::{CardName, CardVariant};
use test_utils::client_interface::{self};
use test_utils::test_adventure::TestAdventure;
use test_utils::*;

const EXAMPLE_CARD: CardVariant = CardVariant::standard(CardName::TestSpell);

#[test]
fn test_initiate_draft() {
    let mut adventure = TestAdventure::new(Side::Champion).build();

    let draft = adventure.insert_tile(TileEntity::Draft(DraftData {
        context: None,
        choices: vec![CardChoice { quantity: 2, card: EXAMPLE_CARD, cost: Coins(0), sold: false }],
    }));

    adventure.visit_tile(draft);

    assert!(adventure.has(Button::DraftPick));
}

#[test]
fn test_pick_card() {
    let mut adventure = TestAdventure::new(Side::Champion).build();

    let draft = adventure.insert_tile(TileEntity::Draft(DraftData {
        context: None,
        choices: vec![CardChoice { quantity: 2, card: EXAMPLE_CARD, cost: Coins(0), sold: false }],
    }));

    adventure.visit_tile(draft);
    adventure.click(Button::DraftPick);
    assert_eq!(adventure.open_panel_count(), 0);
    adventure.click(Button::ShowDeck);
    assert_eq!(adventure.open_panel_count(), 1);

    client_interface::assert_has_element_name(
        adventure.user.interface.top_panel(),
        element_names::deck_card(EXAMPLE_CARD),
    );
}
