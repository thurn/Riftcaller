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
use core_data::adventure_primitives::Coins;
use core_data::game_primitives::Side;
use game_data::card_name::{CardName, CardVariant};
use game_data::card_set_name::CardSetName;
use test_utils::client_interface::{self};
use test_utils::test_adventure::TestAdventure;
use test_utils::*;

// Cost will always be 50 because we use a deterministic random number
// generator.
const BUY_COST: Coins = Coins(50);
const CARD: CardVariant = CardVariant::standard(CardName::TestSingletonSetSpell);

#[test]
fn test_visit_shop() {
    let mut adventure =
        TestAdventure::new(Side::Riftcaller).card_set(CardSetName::TestSingletonSpellSet).build();
    let shop = adventure.insert_tile(AdventureEffect::Shop(CardSelector::default()));
    adventure.visit_tile(shop);
    assert!(adventure.has_text(BUY_COST.to_string()));
}

#[test]
fn test_buy_card() {
    let mut adventure =
        TestAdventure::new(Side::Riftcaller).card_set(CardSetName::TestSingletonSpellSet).build();
    let shop = adventure.insert_tile(AdventureEffect::Shop(CardSelector::default()));
    adventure.visit_tile(shop);

    assert!(adventure.has_text(test_constants::STARTING_COINS.to_string()));
    adventure.click_on(adventure.user_id(), BUY_COST.to_string());
    assert!(adventure.has_text((test_constants::STARTING_COINS - BUY_COST).to_string()));

    adventure.click(Button::CloseIcon);
    adventure.click(Button::ShowDeck);

    client_interface::assert_has_element_name(
        adventure.client.interface.top_panel(),
        element_names::deck_card(CARD),
    );
}
