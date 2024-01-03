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

use adventure_data::adventure::{AdventureState, CardChoice, DraftData, ShopData};
use adventure_data::card_filter_data::{CardFilterCategoryOperator, UpgradedStatus};
use card_definition_data::cards;
use core_data::adventure_primitives::{CardFilterId, Coins};
use enumset::{EnumSet, EnumSetType};
use game_data::card_name::CardVariant;
use game_data::deck::Deck;

/// Cards in the player's deck which match this [CardFilterId].
pub fn deck(deck: &Deck, filter: CardFilterId) -> impl Iterator<Item = CardVariant> + '_ {
    deck.cards.keys().filter(move |&&variant| matches(filter, variant)).copied()
}

/// All possible cards for the current adventure which match this
/// [CardFilterId].
pub fn all_cards(
    state: &AdventureState,
    filter: CardFilterId,
) -> impl Iterator<Item = CardVariant> + '_ {
    cards::all_cards()
        .filter(move |definition| {
            definition.sets.contains(&state.config.card_set)
                && definition.side == state.side
                && matches(filter, definition.variant())
        })
        .map(|definition| definition.variant())
}

/// Builds a standard [DraftData] set of draft choices for the provided
/// [CardFilterId].
pub fn draft_choices(state: &mut AdventureState, filter: CardFilterId) -> DraftData {
    let cards: Vec<_> = all_cards(state, filter).collect();
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
/// [CardFilterId].
pub fn shop_choices(state: &mut AdventureState, filter: CardFilterId) -> ShopData {
    let cards: Vec<_> = all_cards(state, filter).collect();
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
/// [CardFilterId].
pub fn matches(filter_id: CardFilterId, variant: CardVariant) -> bool {
    let filter = game_tables::card_filter(filter_id);
    let definition = cards::get(variant);

    let rarity = check_set(filter.rarity, definition.rarity);
    let types = check_set(filter.card_types, definition.card_type);
    let subtypes = (!filter.card_subtypes.is_empty())
        .then(|| filter.card_subtypes.iter().any(|subtype| definition.subtypes.contains(&subtype)));
    let upgraded = check_set(
        filter.upgraded,
        if definition.config.metadata.is_upgraded {
            UpgradedStatus::Upgraded
        } else {
            UpgradedStatus::Default
        },
    );
    let sides = check_set(filter.sides, definition.side);
    let result =
        vec![rarity, types, subtypes, upgraded, sides].into_iter().flatten().collect::<Vec<_>>();

    match filter.operator {
        CardFilterCategoryOperator::And => result.iter().all(|v| *v),
        CardFilterCategoryOperator::Or => result.iter().any(|v| *v),
    }
}

fn check_set<T: EnumSetType>(set: EnumSet<T>, value: T) -> Option<bool> {
    (!set.is_empty()).then(|| set.contains(value))
}
