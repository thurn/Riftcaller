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

use adventure_data::adventure::{AdventureState, CardChoice, CardSelector, DraftData, ShopData};
use core_data::adventure_primitives::Coins;
use game_data::card_name::CardVariant;
use game_data::deck::Deck;

/// Cards in the player's deck which match this [CardSelector].
pub fn deck(deck: &Deck, selector: CardSelector) -> impl Iterator<Item = CardVariant> + '_ {
    deck.cards.keys().filter(move |&&variant| matches(selector.clone(), variant)).copied()
}

/// All possible cards for the current adventure which match this
/// [CardSelector].
pub fn all_cards(
    state: &AdventureState,
    selector: CardSelector,
) -> impl Iterator<Item = CardVariant> + '_ {
    rules::all_cards()
        .filter(move |definition| {
            definition.sets.contains(&state.config.card_set)
                && definition.side == state.side
                && matches(selector.clone(), definition.variant())
        })
        .map(|definition| definition.variant())
}

/// Builds a standard [DraftData] set of draft choices for the provided
/// [CardSelector].
pub fn draft_choices(state: &mut AdventureState, selector: CardSelector) -> DraftData {
    let cards: Vec<_> = all_cards(state, selector).collect();
    DraftData {
        context: None,
        choices: state
            .config
            .choose_multiple(3, cards.into_iter())
            .into_iter()
            .map(|variant| CardChoice { quantity: 3, card: variant, cost: Coins(0), sold: false })
            .collect(),
    }
}

/// Builds a standard [ShopData] set of shop choices for the provided
/// [CardSelector].
pub fn shop_choices(state: &mut AdventureState, selector: CardSelector) -> ShopData {
    let cards: Vec<_> = all_cards(state, selector).collect();
    ShopData {
        choices: state
            .config
            .choose_multiple(5, cards.into_iter())
            .into_iter()
            .map(|name| CardChoice {
                quantity: state.config.gen_range(1..=3),
                card: name,
                cost: Coins(state.config.gen_range(1..=4) * 25),
                sold: false,
            })
            .collect(),
    }
}

/// Returns true if the specified [CardVariant] is selected by the provided
/// [CardSelector].
pub fn matches(selector: CardSelector, variant: CardVariant) -> bool {
    let definition = rules::get(variant);

    let mut result = definition.rarity >= selector.minimum_rarity;
    result &= (definition.config.metadata.is_upgraded && selector.upgraded)
        || (!definition.config.metadata.is_upgraded && !selector.upgraded);

    if !selector.card_types.is_empty() {
        result &= selector.card_types.contains(definition.card_type);
    }
    if !selector.card_subtypes.is_empty() {
        result &=
            selector.card_subtypes.iter().any(|subtype| definition.subtypes.contains(&subtype));
    }

    result
}
