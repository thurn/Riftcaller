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

use serde::{Deserialize, Serialize};

use crate::adventure::CardSelector;

/// A modification to a specific card in a player's deck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeckCardEffect {
    /// Duplicate this card until the deck contains `count` copies of it.
    Duplicate(u32),
}

/// A modification to the state of an ongoing adventure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdventureEffect {
    /// Show a draft screen to select a card from a list of random choices
    Draft(CardSelector),
    /// Pick a card in the player's deck to apply an effect to
    PickCardForEffect(DeckCardEffect),
    /// Open a shop screen to purchase cards from a set of random choices.
    Shop(CardSelector),
}
