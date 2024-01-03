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

use core_data::game_primitives::{
    AbilityId, AbilityIndex, CardId, CardSubtype, CardType, Rarity, School, Side, Sprite,
};
use game_data::card_configuration::{Ability, CardConfig, Cost};
use game_data::card_name::{CardName, CardVariant};
use game_data::card_set_name::CardSetName;

/// The fundamental object defining the behavior of a given card in Riftcaller
///
/// This struct's top-level fields should be universal properties which need to
/// be set by every card
#[derive(Debug)]
pub struct CardDefinition {
    pub name: CardName,
    pub sets: Vec<CardSetName>,
    pub cost: Cost<CardId>,
    pub image: Sprite,
    pub card_type: CardType,
    pub subtypes: Vec<CardSubtype>,
    pub side: Side,
    pub school: School,
    pub rarity: Rarity,
    pub abilities: Vec<Ability>,
    pub config: CardConfig,
}

impl CardDefinition {
    pub fn variant(&self) -> CardVariant {
        CardVariant { name: self.name, metadata: self.config.metadata }
    }

    /// Returns the ability at the given index. Panics if no ability with this
    /// index exists.
    pub fn ability(&self, index: AbilityIndex) -> &Ability {
        &self.abilities[index.value()]
    }

    /// Iterator over all [AbilityId]s of a card.
    pub fn ability_ids(&self, card_id: CardId) -> impl Iterator<Item = AbilityId> {
        (0..self.abilities.len()).map(move |i| AbilityId::new(card_id, i))
    }

    pub fn is_spell(&self) -> bool {
        self.card_type.is_spell()
    }

    pub fn is_scheme(&self) -> bool {
        self.card_type == CardType::Scheme
    }

    pub fn is_minion(&self) -> bool {
        self.card_type == CardType::Minion
    }

    pub fn is_artifact(&self) -> bool {
        self.card_type == CardType::Artifact
    }

    pub fn is_ally(&self) -> bool {
        self.card_type == CardType::Ally
    }

    pub fn is_permanent(&self) -> bool {
        !self.is_spell()
    }

    pub fn is_infernal(&self) -> bool {
        self.config.resonance.map(|r| r.infernal) == Some(true)
    }

    pub fn is_mortal(&self) -> bool {
        self.config.resonance.map(|r| r.mortal) == Some(true)
    }

    pub fn is_astral(&self) -> bool {
        self.config.resonance.map(|r| r.astral) == Some(true)
    }
}
