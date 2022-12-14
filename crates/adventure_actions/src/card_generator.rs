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

use data::adventure::{AdventureState, DraftChoice, DraftData};
use data::card_name::CardName;
use data::primitives::Rarity;
use data::set_name::SetName;

/// Generates options for drafting a card during an adventure
pub fn draft_choices(state: &mut AdventureState) -> DraftData {
    DraftData {
        choices: state
            .choose_multiple(3, common_cards())
            .into_iter()
            .map(|name| DraftChoice { quantity: 1, card: name })
            .collect(),
    }
}

fn common_cards() -> impl Iterator<Item = CardName> {
    rules::CARDS
        .iter()
        .filter(|(_, definition)| {
            definition.sets.contains(&SetName::Core2024) && definition.rarity == Rarity::Common
        })
        .map(|(name, _)| *name)
}
