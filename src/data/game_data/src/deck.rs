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

//! Defines a deck as it exists outside of an active game

use std::collections::HashMap;
use std::iter;

use core_data::game_primitives::{School, Side};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::card_name::CardVariant;

/// Represents a player deck outside of an active game
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    /// Identifies which side this  deck plays as.
    pub side: Side,
    /// The schools associated with this deck, in order of selection. The first
    /// school is often called the 'primary' school for a deck and is e.g. used
    /// to determine the card backs shown for this player.
    pub schools: Vec<School>,
    /// Identity cards for this deck. Currently you can only ever have one
    /// identity card, but who knows.
    pub identities: Vec<CardVariant>,
    /// Sigil cards for this deck, which start the game in play and provide
    /// global effects.
    pub sigils: Vec<CardVariant>,
    /// How many (non-identity, non-sigil) cards with each name are present in
    /// this deck?
    #[serde_as(as = "Vec<(_, _)>")]
    pub cards: HashMap<CardVariant, u32>,
}

impl Deck {
    /// Returns a vector which repeats each [CardVariant] in [Self::cards] in
    /// alphabetical order a number of times equal to its deck count. Note: The
    /// returned vector does *not* contain identity or sigil cards.
    pub fn card_variants(&self) -> Vec<CardVariant> {
        let mut result = self
            .cards
            .iter()
            .flat_map(|(name, count)| iter::repeat(*name).take(*count as usize))
            .collect::<Vec<_>>();
        result.sort();
        result
    }
}
