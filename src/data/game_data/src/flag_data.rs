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

use core_data::game_primitives::{AbilityId, HasAbilityId};

/// A Flag is a variant of boolean which typically indicates whether some game
/// action can currently be taken. Flags have a 'default' state, which is the
/// value of the flag based on standard game rules, and an 'override' state,
/// which is a value set by specific delegates. An override of 'false' takes
/// precedence over an override of 'true'.
///
/// For example, the 'CanPlay' delegate will be invoked with
/// `Flag::Default(false)` if a card cannot currently be played according to the
/// standard game rules (sufficient mana available, correct player's turn, etc).
/// A delegate could transform this via `allow()` to allow the card
/// to be played. A second delegate could set `disallow()` to prevent
/// the card from being played, and this would take priority.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Flag {
    /// Initial value of this flag
    Default(bool),
    /// Override for this flag set by a delegate.
    Override(bool),
}

impl Flag {
    pub fn new(value: bool) -> Self {
        Self::Default(value)
    }

    /// Allows some player action or event that would not otherwise happen. This
    /// has priority over base game rules, but is superseded in turn by
    /// [Self::disallow] and [Self::add_constraint].
    pub fn allow(self) -> Self {
        self.override_unconditionally(true)
    }

    /// Prevents some player action or event from happening. This is the highest
    /// priority option and cannot be superseded.
    pub fn disallow(self) -> Self {
        self.override_unconditionally(false)
    }

    /// Overrides this flag if `value` is false. This is used to modify
    /// something that a player *could otherwise do* with an additional
    /// constraint that prevents it from happening. It cannot *expand* the
    /// scope where an event can happen.
    pub fn add_constraint(self, value: bool) -> Self {
        if value {
            self
        } else {
            self.override_unconditionally(value)
        }
    }

    /// Overrides this flag if `value` is true. This is used to modify
    /// something that a player *could not* otherwise do with an additional
    /// capability. It expands the scope of where an action can happen, but
    /// cannot *restrict* anything that was already allowed.
    ///
    /// This has lower priority than [Self::add_constraint]. This behavior is
    /// sometimes described as the "can't beats can" rule.
    pub fn add_permission(self, value: bool) -> Self {
        if value {
            self.override_unconditionally(value)
        } else {
            self
        }
    }

    fn override_unconditionally(self, value: bool) -> Self {
        match self {
            Self::Default(_) => Self::Override(value),
            Self::Override(current) => Self::Override(current && value),
        }
    }
}

impl From<Flag> for bool {
    fn from(flag: Flag) -> Self {
        match flag {
            Flag::Default(value) | Flag::Override(value) => value,
        }
    }
}

/// An AbilityFlag is a [Flag] which keeps track of which [AbilityId] caused the
/// flag value to change. See [Flag] for full documentation.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum AbilityFlag {
    Default(bool),
    Override(bool, AbilityId),
}

impl AbilityFlag {
    pub fn new(value: bool) -> Self {
        Self::Default(value)
    }

    pub fn value(self) -> bool {
        match self {
            AbilityFlag::Default(value) | AbilityFlag::Override(value, _) => value,
        }
    }

    pub fn ability_id(self) -> Option<AbilityId> {
        match self {
            AbilityFlag::Override(_, ability_id) => Some(ability_id),
            _ => None,
        }
    }

    /// See [Flag::allow].
    pub fn allow(self, ability: impl HasAbilityId) -> Self {
        self.override_unconditionally(true, ability.ability_id())
    }

    /// See [Flag::disallow].
    pub fn disallow(self, ability: impl HasAbilityId) -> Self {
        self.override_unconditionally(false, ability.ability_id())
    }

    /// See [Flag::add_constraint].
    pub fn add_constraint(self, value: bool, ability: impl HasAbilityId) -> Self {
        if value {
            self
        } else {
            self.override_unconditionally(value, ability.ability_id())
        }
    }

    /// See [Flag::add_permission].
    pub fn add_permission(self, value: bool, ability: impl HasAbilityId) -> Self {
        if value {
            self.override_unconditionally(value, ability.ability_id())
        } else {
            self
        }
    }

    fn override_unconditionally(self, value: bool, ability_id: AbilityId) -> Self {
        match self {
            Self::Default(_) => Self::Override(value, ability_id),
            Self::Override(current, _) if current && !value => Self::Override(false, ability_id),
            _ => self,
        }
    }
}
