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
use data::adventure::Coins;
use data::primitives::Side;
use test_utils::client_interface::HasText;
use test_utils::test_adventure::{TestAdventure, EXPLORE_ICON};

#[test]
fn test_open_explore_panel() {
    let mut adventure = TestAdventure::new(Side::Champion);
    adventure.visit_tile_with_icon(EXPLORE_ICON);
    assert!(adventure.interface.top_panel().has_text("Explore"));
}

#[test]
fn test_close_explore_panel() {
    let mut adventure = TestAdventure::new(Side::Champion);
    adventure.visit_tile_with_icon(EXPLORE_ICON);
    adventure.click_on("Close");
    assert_eq!(adventure.interface.panel_count(), 0);
}

#[test]
fn test_invoke_explore() {
    let mut adventure = TestAdventure::new(Side::Champion);
    adventure.visit_tile_with_icon(EXPLORE_ICON);
    let count = adventure.map.tile_count();
    adventure.click_on(format!("Explore: 100 {}", icons::COINS));
    assert!(adventure.map.tile_count() > count);
    assert!(adventure
        .interface
        .screen_overlay()
        .has_text(format!("{}", /* adventure_generator::STARTING_COINS - */ Coins(400))));
}
