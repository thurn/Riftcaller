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

use core_data::adventure_primitives::Coins;
use game_data::card_name::CardName;
use serde::{Deserialize, Serialize};

use crate::adventure::CardSelector;

/// A modification to a specific card in a player's deck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeckCardEffect {
    /// Duplicate this card until the deck contains 3 copies of it
    DuplicateTo3Copies,
    /// Transform all copies of a card into another randomly-chosen card of a
    /// higher rarity
    TransmuteAllCopies,
    /// Upgrade all copies of card, adding special 'upgraded' text to it
    UpgradeAllCopies,
    /// Remove one copy of a card from the player's deck
    RemoveOne,
}

/// A modification to the state of an ongoing adventure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdventureEffect {
    /// Gain a quantity of coins
    GainCoins(Coins),
    /// Lose coins. This choice cannot be selected if insufficient coins are
    /// available.
    LoseCoins(Coins),
    /// Lose all coins
    LoseAllCoins,
    /// Gain a quantity of arcanite
    GainArcanite(u32),
    /// Show a draft screen to select a card from a list of random choices
    Draft(CardSelector),
    /// Open a shop screen to purchase cards from a set of random choices.
    Shop(CardSelector),
    /// The player may pick a card in their deck matching [CardSelector] to
    /// apply [DeckCardEffect] to.
    PickCardForEffect(CardSelector, DeckCardEffect),
    /// Modify a random card in the player's deck matching this [CardSelector]
    /// by applying a [DeckCardEffect] to it. The card chosen is known to the
    /// player in advance.
    KnownRandomCardEffect(CardSelector, DeckCardEffect),
    /// Modify a random card in the player's deck matching this [CardSelector]
    /// by applying a [DeckCardEffect] to it. The card chosen is not known
    /// to the player in advance.
    UnknownRandomCardEffect(CardSelector, DeckCardEffect),
    /// Apply a [DeckCardEffect] to all cards matching this [CardSelector].
    ApplyCardEffectToAllMatching(CardSelector, DeckCardEffect),
    /// Add a quantity of random additional tiles to the world map
    AddMapTiles(u32),
    /// Add a quantity of standard draft tiles to the world map
    AddDraftTiles(u32),
    /// Add a quantity of standard narrative event tiles to the world map
    AddNarrativeTiles(u32),
    /// Add a known fixed card to the player's deck. The card is always the same
    /// for this narrative event.
    GainKnownFixedCard(CardName),
    /// Add a known random card to the player's deck matching this
    /// [CardSelector]. The card received is known to the player in advance.
    GainKnownRandomCard(CardSelector),
    /// Add an unknown random card to the player's deck matching this
    /// [CardSelector]. The card received is not known to the player in advance.
    GainUnknownRandomCard(CardSelector),
    /// Pick a card from the deck for the player to lose, matching this
    /// [CardSelector].
    ///
    /// "Losing" a card is treated as a cost, unlike "removing" a card via
    /// [DeckCardEffect], meaning that this option cannot be selected if a
    /// matching card is not available.    
    PickCardToLose(CardSelector),
    /// Lose a known random card from the player's deck matching this
    /// [CardSelector]. The card lost is known to the player in advance.
    ///
    /// "Losing" a card is treated as a cost, unlike "removing" a card via
    /// [DeckCardEffect], meaning that this option cannot be selected if a
    /// matching card is not available.
    LoseKnownRandomCard(CardSelector),
    /// Lose an unknown random card from the player's deck matching this
    /// [CardSelector]. The card lost is not known to the player in advance.
    ///
    /// "Losing" a card is treated as a cost, unlike "removing" a card via
    /// [DeckCardEffect], meaning that this option cannot be selected if a
    /// matching card is not available.
    LoseUnknownRandomCard(CardSelector),
}

/// State for an [AdventureEffect]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdventureEffectData {
    /// The effect
    pub effect: AdventureEffect,
    /// Describes the result of evaluating this effect
    pub description: String,
    /// Optionally, a known card name associated with this effect.
    ///
    /// For "Known" effects like [AdventureEffect::GainKnownRandomCard], the
    /// card selected is picked in advance and stored here so that the player
    /// can see the outcome before acting.
    pub known_card: Option<CardName>,
}
