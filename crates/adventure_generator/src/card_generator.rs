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

use data::adventure::{AdventureConfiguration, CardChoice, Coins, DraftData, ShopData};
use data::card_name::CardName;
use data::primitives::{Rarity, Side};
use data::set_name::SetName;

/// Generates options for drafting a card during an adventure
pub fn draft_choices(config: &mut AdventureConfiguration) -> DraftData {
    DraftData {
        choices: config
            .choose_multiple(3, common_cards(config.side))
            .into_iter()
            .map(|name| CardChoice { quantity: 1, card: name, cost: Coins(0), sold: false })
            .collect(),
    }
}

/// Generates options for buying from a shop during an adventure
pub fn shop_options(config: &mut AdventureConfiguration) -> ShopData {
    ShopData {
        visited: false,
        choices: config
            .choose_multiple(5, common_cards(config.side))
            .into_iter()
            .map(|name| CardChoice {
                quantity: config.gen_range(1..=3),
                card: name,
                cost: Coins(config.gen_range(1..=4) * 25),
                sold: false,
            })
            .collect(),
    }
}

fn common_cards(side: Side) -> impl Iterator<Item = CardName> {
    rules::all_cards()
        .filter(move |definition| {
            definition.sets.contains(&SetName::Core2024)
                && definition.rarity == Rarity::Common
                && definition.side == side
        })
        .map(|definition| definition.name)
}
