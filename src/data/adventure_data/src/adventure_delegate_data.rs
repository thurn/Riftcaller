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

use std::fmt;

use anyhow::Result;
use core_data::adventure_primitives::{AdventureAbilityId, AdventureCardId};
use game_data::card_name::CardMetadata;

use crate::adventure::AdventureState;

/// Identifies the context for a given request to a delegate: which player,
/// card, & card ability owns the delegate.
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct AdventureScope {
    /// Ability which owns this delegate.
    ability_id: AdventureAbilityId,
    /// Metadata for this card
    metadata: CardMetadata,
}

impl AdventureScope {
    pub fn new(ability_id: AdventureAbilityId, metadata: CardMetadata) -> Self {
        Self { ability_id, metadata }
    }

    /// Ability which owns this scope
    pub fn ability_id(&self) -> AdventureAbilityId {
        self.ability_id
    }

    /// Card which owns this scope
    pub fn card_id(&self) -> AdventureCardId {
        self.ability_id.card_id
    }

    pub fn metadata(&self) -> CardMetadata {
        self.metadata
    }

    pub fn is_upgraded(&self) -> bool {
        self.metadata.is_upgraded
    }

    /// Returns one of two values based on whether the card is upgraded
    pub fn upgrade<T>(&self, normal: T, upgraded: T) -> T {
        self.metadata.upgrade(normal, upgraded)
    }
}

impl fmt::Debug for AdventureScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.ability_id)
    }
}

/// Predicate to determine whether a delegate should run, taking contextual
/// information `T`.
pub type AdventureRequirement<T> = fn(&AdventureState, AdventureScope, &T) -> bool;
/// Function to mutate game state in response to an event, taking contextual
/// information `T`.
pub type AdventureMutation<T> = fn(&mut AdventureState, AdventureScope, &T) -> Result<()>;
/// Function to intercept a query for game information, taking contextual
/// information `T` and the current query value `R`.
pub type AdventureTransformation<T, R> = fn(&AdventureState, AdventureScope, &T, R) -> R;

/// Delegate which responds to a given game event and mutates game state in
/// response.
#[derive(Copy, Clone)]
pub struct AdventureEvent<T> {
    /// Should return true if this delegate's `mutation` should run.
    pub requirement: AdventureRequirement<T>,
    /// Modifies the current adventure state in response to the associated
    /// event.
    pub mutation: AdventureMutation<T>,
}

impl<T> AdventureEvent<T> {
    pub fn new(requirement: AdventureRequirement<T>, mutation: AdventureMutation<T>) -> Self {
        Self { requirement, mutation }
    }
}

/// Delegate which intercepts and transforms a query for game information.
#[derive(Copy, Clone)]
pub struct AdventureQuery<T, R> {
    /// Should return true if this delegate's `transformation` should run.
    pub requirement: AdventureRequirement<T>,
    /// Function which takes contextual data and the current value of some piece
    /// of game information and returns a transformed value for this
    /// information.
    pub transformation: AdventureTransformation<T, R>,
}

impl<T, R> AdventureQuery<T, R> {
    pub fn new(
        requirement: AdventureRequirement<T>,
        transformation: AdventureTransformation<T, R>,
    ) -> Self {
        Self { requirement, transformation }
    }
}
