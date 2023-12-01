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

use core_data::game_primitives::Side;
use core_ui::prelude::Node;
use element_names::ElementName;
use game_data::card_name::{CardName, CardVariant};
use test_utils::client_interface::{self, find_card_view, HasText};
use test_utils::test_adventure::TestAdventure;
use test_utils::test_session::TestSession;
use test_utils::*;

const EXAMPLE_CARD: CardName = CardName::TestSpell;

#[test]
fn test_open_deck_editor() {
    let mut adventure = TestAdventure::new(Side::Champion).build();
    adventure.click(Button::ShowDeck);

    client_interface::assert_has_element_name(
        adventure.client.interface.top_panel(),
        element_names::COLLECTION_BROWSER,
    );
    client_interface::assert_has_element_name(
        adventure.client.interface.top_panel(),
        element_names::CARD_LIST,
    );
}

#[test]
fn test_remove_from_deck() {
    let mut adventure = TestAdventure::new(Side::Champion)
        .deck_card(EXAMPLE_CARD, 2)
        .collection_card(EXAMPLE_CARD, 3)
        .build();
    adventure.click(Button::ShowDeck);

    let quantity1 = find_card_node(&adventure, element_names::card_list_card_quantity);
    assert_eq!("2x", quantity1.all_text());

    let draggable = client_interface::find_draggable(find_card_node(
        &adventure,
        element_names::card_list_card_name,
    ))
    .expect("Draggable node");

    adventure.perform(
        draggable.on_drop.clone().expect("Drop action").action.expect("action"),
        adventure.user_id(),
    );

    let quantity2 = find_card_node(&adventure, element_names::card_list_card_quantity);
    assert_eq!("1x", quantity2.get_text().join(""));

    let card = find_card_node(&adventure, element_names::deck_card);
    // Quantity in collection browser is unchanged
    assert_eq!("3x", find_card_quantity(card).expect("quantity"));
}

#[test]
fn test_add_to_deck() {
    let mut adventure = TestAdventure::new(Side::Champion)
        .deck_card(EXAMPLE_CARD, 2)
        .collection_card(EXAMPLE_CARD, 3)
        .build();
    adventure.click(Button::ShowDeck);

    let quantity1 = find_card_node(&adventure, element_names::card_list_card_quantity);
    assert_eq!("2x", quantity1.all_text());

    let draggable =
        client_interface::find_draggable(find_card_node(&adventure, element_names::deck_card_slot))
            .expect("Draggable node");

    adventure.perform(
        draggable.on_drop.clone().expect("Drop action").action.expect("action"),
        adventure.user_id(),
    );

    let quantity2 = find_card_node(&adventure, element_names::card_list_card_quantity);
    assert_eq!("3x", quantity2.all_text());

    assert!(find_card_node(&adventure, element_names::deck_card_slot_overlay).has_text("In Deck"));
    let card = find_card_node(&adventure, element_names::deck_card);

    // Quantity in collection browser is unchanged
    assert_eq!("3x", find_card_quantity(card).expect("quantity"));
}

fn find_card_quantity(node: &Node) -> Option<&String> {
    let view = find_card_view(node).expect("CardView");
    view.card_icons.as_ref()?.top_right_icon.as_ref()?.text.as_ref()
}

fn find_card_node(adventure: &TestSession, f: impl Fn(CardVariant) -> ElementName) -> &Node {
    client_interface::find_element_name(
        adventure.client.interface.top_panel(),
        f(CardVariant::standard(EXAMPLE_CARD)),
    )
    .expect("Node not found")
}
