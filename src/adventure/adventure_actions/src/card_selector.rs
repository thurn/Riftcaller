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

use adventure_data::adventure::{AdventureState, CardSelector};
use game_data::card_name::{CardName, CardVariant};

/// Names of cards in the player's deck which match this [CardSelector].
pub fn for_deck<'a>(
    adventure: &'a AdventureState,
    selector: &'a CardSelector,
) -> impl Iterator<Item = CardName> + 'a {
    adventure
        .deck
        .cards
        .keys()
        .filter(|&&variant| matches(selector, variant))
        .map(|variant| variant.name)
}

/// Returns true if the specified [CardVariant] is selected by the provided
/// [CardSelector].
pub fn matches(selector: &CardSelector, variant: CardVariant) -> bool {
    let mut result = true;
    let definition = rules::get(variant);

    if let Some(rarity) = selector.rarity {
        result &= definition.rarity >= rarity;
    }
    if !selector.card_types.is_empty() {
        result &= selector.card_types.contains(&definition.card_type);
    }
    if !selector.card_subtypes.is_empty() {
        result &=
            selector.card_subtypes.iter().any(|subtype| definition.subtypes.contains(subtype));
    }

    result
}
