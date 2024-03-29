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

use std::collections::HashMap;

use core_data::game_primitives::AbilityId;
use dashmap::DashSet;
use game_data::card_name::{CardMetadata, CardVariant};
use game_data::card_state::CardState;
use game_data::game_state::GameState;
use once_cell::sync::Lazy;

use crate::ability_data::Ability;
use crate::card_definition::CardDefinition;

pub type CardFn = fn(CardMetadata) -> CardDefinition;

pub static DEFINITIONS: Lazy<DashSet<CardFn>> = Lazy::new(DashSet::new);

/// Contains [CardDefinition]s for all known cards, keyed by [CardVariant]
static CARDS: Lazy<HashMap<CardVariant, CardDefinition>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for card_fn in DEFINITIONS.iter() {
        for upgraded in [false, true] {
            let metadata = CardMetadata { is_upgraded: upgraded };
            let mut card = card_fn(metadata);
            card.config.metadata = metadata;
            let variant = CardVariant { name: card.name, metadata };
            assert!(!map.contains_key(&variant), "Duplicate card name found");
            map.insert(variant, card);
        }
    }
    map
});

/// Returns an iterator over all known card definitions in an undefined order
pub fn all_cards() -> impl Iterator<Item = &'static CardDefinition> {
    assert!(CARDS.len() > 0, "Card not found. Call initialize() or update cards?");
    CARDS.values()
}

/// Looks up the definition for a [CardVariant]. Panics if no such card is
/// defined. If this panics, you are probably not calling initialize::run();
pub fn get(variant: CardVariant) -> &'static CardDefinition {
    CARDS
        .get(&variant)
        .unwrap_or_else(|| panic!("Card not found. Call initialize() or update cards?"))
}

pub fn ability_definition(game: &GameState, ability_id: AbilityId) -> &'static Ability {
    game.card(ability_id.card_id).definition().ability(ability_id.index)
}

pub trait CardDefinitionExt {
    fn definition(&self) -> &'static CardDefinition;
}

impl CardDefinitionExt for CardState {
    fn definition(&self) -> &'static CardDefinition {
        get(self.variant)
    }
}
