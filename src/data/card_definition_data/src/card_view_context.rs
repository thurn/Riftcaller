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

use core_data::game_primitives::CardId;
use game_data::card_state::CardState;
use game_data::game_state::GameState;

use crate::card_definition::CardDefinition;

/// Provides the context in which a card view is being displayed, i.e. either
/// during an active game or in a deck editor.
pub enum CardViewContext<'a> {
    Default(&'a CardDefinition),
    Game(&'a CardDefinition, &'a GameState, &'a CardState),
}

impl<'a> CardViewContext<'a> {
    pub fn definition(&self) -> &CardDefinition {
        match self {
            Self::Default(d) => d,
            Self::Game(d, _, _) => d,
        }
    }

    pub fn game(&self) -> Option<&GameState> {
        match self {
            Self::Default(_) => None,
            Self::Game(_, game, _) => Some(&game),
        }
    }

    pub fn card(&self) -> Option<&CardState> {
        match self {
            Self::Default(_) => None,
            Self::Game(_, _, card) => Some(&card),
        }
    }

    /// Invokes the provided `game` function to produce a value in the active
    /// game context, otherwise returns some `default`.
    pub fn query_or<T>(&self, default: T, fun: impl Fn(&GameState, &CardState) -> T) -> T {
        match self {
            Self::Default(_) => default,
            Self::Game(_, state, card) => fun(state, card),
        }
    }

    /// Equivalent to `query_or` which uses `None` as the default value.
    pub fn query_or_none<T>(&self, fun: impl Fn(&GameState, &CardState) -> T) -> Option<T> {
        match self {
            Self::Default(_) => None,
            Self::Game(_, state, card) => Some(fun(state, card)),
        }
    }

    /// Equivalent to `query_or` which passed the [CardId] to the callback
    /// function.
    pub fn query_id_or<T>(&self, default: T, fun: impl Fn(&GameState, CardId) -> T) -> T {
        match self {
            Self::Default(_) => default,
            Self::Game(_, state, card) => fun(state, card.id),
        }
    }

    /// Equivalent to `query_or` which accepts a function returning
    /// [anyhow::Result].
    pub fn query_or_ok<T>(
        &self,
        default: T,
        fun: impl Fn(&GameState, &CardState) -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        match self {
            Self::Default(_) => Ok(default),
            Self::Game(_, state, card) => fun(state, card),
        }
    }
}
