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
use data::adventure::{CardChoice, Coins, DraftData, TileEntity};
use data::card_name::CardName;
use data::primitives::Side;
use test_utils::client_interface::{self, HasText};
use test_utils::test_adventure::{TestAdventure, TestConfig, DRAFT_ICON};

const DRAFT_COST: Coins = Coins(25);
const EXAMPLE_CARD: CardName = CardName::TestChampionSpell;

#[test]
fn test_initiate_draft() {
    let mut adventure = TestAdventure::new(Side::Champion, config());
    adventure.visit_tile_with_icon(DRAFT_ICON);
    assert!(adventure.interface.top_panel().has_text("An expedition"));

    assert!(adventure
        .interface
        .screen_overlay()
        .has_text(format!("{}", adventure_generator::STARTING_COINS)));

    adventure.click_on("Draft");

    assert!(adventure
        .interface
        .screen_overlay()
        .has_text(format!("{}", adventure_generator::STARTING_COINS - DRAFT_COST)));
    assert!(adventure.interface.top_panel().has_text("Pick"));
}

#[test]
fn test_pick_card() {
    let mut adventure = TestAdventure::new(Side::Champion, config());

    adventure.visit_tile_with_icon(DRAFT_ICON);
    adventure.click_on("Draft");
    adventure.click_on("Pick");
    assert_eq!(adventure.interface.panel_count(), 0);
    adventure.click_on_navbar(icons::DECK);

    client_interface::assert_has_element_name(
        adventure.interface.top_panel(),
        element_names::deck_card(EXAMPLE_CARD),
    );
}

fn config() -> TestConfig {
    TestConfig {
        draft: Some(TileEntity::Draft {
            cost: DRAFT_COST,
            data: DraftData {
                choices: vec![CardChoice {
                    quantity: 2,
                    card: EXAMPLE_CARD,
                    cost: Coins(0),
                    sold: false,
                }],
            },
        }),
        ..TestConfig::default()
    }
}
