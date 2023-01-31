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

use core_ui::icons;
use core_ui::prelude::Node;
use element_names::ElementName;
use game_data::card_name::CardName;
use game_data::deck::Deck;
use game_data::primitives::Side;
use maplit::hashmap;
use test_utils::client_interface::{self, HasText};
use test_utils::test_adventure::{TestAdventure, TestConfig};

const EXAMPLE_CARD: CardName = CardName::TestChampionSpell;

#[test]
fn test_deck_editor_tutorial_prompt() {
    let mut adventure =
        TestAdventure::new(Side::Champion, TestConfig { show_tutorial: true, ..config() });
    adventure.click_on_navbar(icons::DECK);
    assert!(&adventure.interface.top_panel().has_text("Retiring to the library"));
}

#[test]
fn test_open_deck_editor() {
    let mut adventure = TestAdventure::new(Side::Champion, config());
    adventure.click_on_navbar(icons::DECK);
    client_interface::assert_has_element_name(
        adventure.interface.top_panel(),
        element_names::COLLECTION_BROWSER,
    );
    client_interface::assert_has_element_name(
        adventure.interface.top_panel(),
        element_names::CARD_LIST,
    );
}

#[test]
fn test_remove_from_deck() {
    let mut adventure = TestAdventure::new(Side::Champion, config());
    adventure.click_on_navbar(icons::DECK);

    let quantity1 = find_card_node(&adventure, element_names::card_list_card_quantity);
    assert_eq!("2x", quantity1.all_text());

    let draggable = client_interface::find_draggable(find_card_node(
        &adventure,
        element_names::card_list_card_name,
    ))
    .expect("Draggable node");
    adventure.perform_client_action(draggable.on_drop.clone().expect("Drop action"));

    let quantity2 = find_card_node(&adventure, element_names::card_list_card_quantity);
    assert_eq!("1x", quantity2.get_text().join(""));

    // Quantity in collection browser is unchanged
    assert_eq!("3x", find_card_node(&adventure, element_names::deck_card_quantity).all_text());
}

#[test]
fn test_add_to_deck() {
    let mut adventure = TestAdventure::new(Side::Champion, config());
    adventure.click_on_navbar(icons::DECK);

    let quantity1 = find_card_node(&adventure, element_names::card_list_card_quantity);
    assert_eq!("2x", quantity1.all_text());

    let draggable =
        client_interface::find_draggable(find_card_node(&adventure, element_names::deck_card_slot))
            .expect("Draggable node");
    adventure.perform_client_action(draggable.on_drop.clone().expect("Drop action"));

    let quantity2 = find_card_node(&adventure, element_names::card_list_card_quantity);
    assert_eq!("3x", quantity2.all_text());

    assert!(find_card_node(&adventure, element_names::deck_card_slot_overlay).has_text("In Deck"));

    // Quantity in collection browser is unchanged
    assert_eq!("3x", find_card_node(&adventure, element_names::deck_card_quantity).all_text());
}

fn config() -> TestConfig {
    TestConfig {
        deck: Some(Deck {
            side: Side::Champion,
            leader: CardName::TestChampionLeader,
            cards: hashmap! { EXAMPLE_CARD => 2 },
        }),
        collection: hashmap! { EXAMPLE_CARD => 3},
        ..TestConfig::default()
    }
}

fn find_card_node(adventure: &TestAdventure, f: impl Fn(CardName) -> ElementName) -> &Node {
    client_interface::find_element_name(adventure.interface.top_panel(), f(EXAMPLE_CARD))
        .expect("Node")
}
