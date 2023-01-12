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
use data::primitives::Side;
use test_utils::client_interface::{self, HasText};
use test_utils::test_adventure::{TestAdventure, TestConfig};

#[test]
fn test_deck_editor_tutorial_prompt() {
    let mut adventure = TestAdventure::new(Side::Champion, TestConfig { show_tutorial: true });
    adventure.click_on_navbar(icons::DECK);
    assert!(&adventure.interface.top_panel().has_text("Retiring to the library"));
}

#[test]
fn test_open_deck_editor() {
    let mut adventure = TestAdventure::new(Side::Champion, TestConfig::default());
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
