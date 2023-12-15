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

use adventure_data::adventure::CardSelector;
use adventure_data::adventure_effect_data::AdventureEffect;
use core_data::game_primitives::Side;
use game_data::card_name::{CardName, CardVariant};
use game_data::card_set_name::CardSetName;
use test_utils::client_interface::{self};
use test_utils::test_adventure::TestAdventure;
use test_utils::*;

const CARD: CardVariant = CardVariant::standard(CardName::TestSingletonSetSpell);

#[test]
fn test_initiate_draft() {
    let mut adventure =
        TestAdventure::new(Side::Riftcaller).card_set(CardSetName::TestSingletonSpellSet).build();
    let draft = adventure.insert_tile(AdventureEffect::Draft(CardSelector::default()));
    adventure.visit_tile(draft);
    assert!(adventure.has(Button::DraftPick));
}

#[test]
fn test_pick_card() {
    let mut adventure =
        TestAdventure::new(Side::Riftcaller).card_set(CardSetName::TestSingletonSpellSet).build();
    let draft = adventure.insert_tile(AdventureEffect::Draft(CardSelector::default()));

    adventure.visit_tile(draft);
    adventure.click(Button::DraftPick);
    assert_eq!(adventure.open_panel_count(), 0);
    adventure.click(Button::ShowDeck);
    assert_eq!(adventure.open_panel_count(), 1);

    client_interface::assert_has_element_name(
        adventure.client.interface.top_panel(),
        element_names::card_list_card_name(CARD),
    );
}
