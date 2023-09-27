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

//! Core functions of the Delegate system. See the module-level comment in
//! `delegate_data` for more information about this system.

use std::fmt::Debug;

use anyhow::Result;
use game_data::card_definition::CardDefinition;
use game_data::card_name::CardMetadata;
use game_data::delegate_data::{DelegateCache, DelegateContext, EventData, QueryData, Scope};
use game_data::game_state::GameState;
use game_data::primitives::{AbilityId, CardId};

/// Adds a [DelegateCache] for this game in order to improve lookup performance.
pub fn populate_delegate_cache(game: &mut GameState) {
    let mut result = DelegateCache::default();
    for card_id in game.all_card_ids() {
        let variant = game.card(card_id).variant;
        let definition = crate::get(variant);
        add_card_to_delegate_cache(&mut result, definition, card_id, variant.metadata);
    }
    game.delegate_cache = result;
}

/// Adds a new card's [CardDefinition] to the delegate cache.
pub fn add_card_to_delegate_cache(
    cache: &mut DelegateCache,
    definition: &CardDefinition,
    card_id: CardId,
    metadata: CardMetadata,
) {
    for (index, ability) in definition.abilities.iter().enumerate() {
        let ability_id = AbilityId::new(card_id, index);
        let scope = Scope::new(ability_id, metadata);
        for delegate in &ability.delegates {
            cache
                .lookup
                .entry(delegate.kind())
                .or_insert_with(Vec::new)
                .push(DelegateContext { delegate: delegate.clone(), scope });
        }
    }
}

/// Removes all cached delegates for a given card.
///
/// This function assumes that the set of delegates for the card has not
/// changed, which is currently always the case.
pub fn remove_card_from_delegate_cache(
    cache: &mut DelegateCache,
    definition: &CardDefinition,
    card_id: CardId,
) {
    for (_, ability) in definition.abilities.iter().enumerate() {
        for delegate in &ability.delegates {
            cache
                .lookup
                .entry(delegate.kind())
                .and_modify(|list| list.retain(|context| context.scope.card_id() != card_id));
        }
    }
}

/// Called when a game event occurs, invokes each registered
/// `Delegate` for this event to mutate the [GameState]
/// appropriately.
pub fn invoke_event<D: Debug, E: EventData<D>>(game: &mut GameState, event: E) -> Result<()> {
    let count = game.delegate_cache.delegate_count(event.kind());
    for i in 0..count {
        let delegate_context = game.delegate_cache.get(event.kind(), i);
        let scope = delegate_context.scope;
        let functions = E::extract(&delegate_context.delegate).expect("Delegate not in cache!");
        let data = event.data();
        if (functions.requirement)(game, scope, data) {
            (functions.mutation)(game, scope, data)?;
        }
    }

    Ok(())
}

/// Called when game state information is needed. Invokes each registered
/// `Delegate` for this query and allows them to intercept &
/// transform the final result.
pub fn perform_query<D: Debug, R: Debug, E: QueryData<D, R>>(
    game: &GameState,
    query: E,
    initial_value: R,
) -> R {
    let mut result = initial_value;
    let count = game.delegate_cache.delegate_count(query.kind());
    for i in 0..count {
        let delegate_context = game.delegate_cache.get(query.kind(), i);
        let scope = delegate_context.scope;
        let functions = E::extract(&delegate_context.delegate).expect("Delegate not in cache!");
        let data = query.data();
        if (functions.requirement)(game, scope, data) {
            result = (functions.transformation)(game, scope, data, result);
        }
    }
    result
}
