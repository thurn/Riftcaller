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

use core_data::game_primitives::{CardSubtype, CardType, Rarity, Side};
use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};

/// Flags for whether or not a card is upgraded
#[derive(Hash, Debug, Serialize, Deserialize, Ord, PartialOrd, EnumSetType)]
pub enum UpgradedStatus {
    Default,
    Upgraded,
}

/// Boolean operators to combine predicates across different [CardFilter]
/// categories.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CardFilterCategoryOperator {
    /// All populated sets must match
    And,
    /// At least one populated set must match
    Or,
}

/// Specifies the parameters for picking a card from a set
///
/// Everything within this struct is expressed as a set of possible options to
/// apply. When one or more of these options are present, only cards that match
/// those flags pass this filter.
///
/// *However* if an option set is empty, it means that all cards match for this
/// attribute. There is no way to specify "none of these options are allowed"
/// and generally this would not make sense.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardFilter {
    /// Operation used to combine the different set categories. See
    /// [CardFilterCategoryOperator].
    pub operator: CardFilterCategoryOperator,
    /// Allowable rarity for cards.
    pub rarity: EnumSet<Rarity>,
    /// Card types to select from.
    pub card_types: EnumSet<CardType>,
    /// Card subtypes to select from.
    pub card_subtypes: EnumSet<CardSubtype>,
    /// Controls whether upgraded cards match this filter.
    pub upgraded: EnumSet<UpgradedStatus>,
    /// Sides that can own these cards
    pub sides: EnumSet<Side>,
}

impl Default for CardFilter {
    fn default() -> Self {
        Self {
            operator: CardFilterCategoryOperator::And,
            rarity: Default::default(),
            card_types: Default::default(),
            card_subtypes: Default::default(),
            upgraded: Default::default(),
            sides: Default::default(),
        }
    }
}

impl CardFilter {
    pub fn new(operator: CardFilterCategoryOperator) -> Self {
        Self { operator, ..CardFilter::default() }
    }
}
