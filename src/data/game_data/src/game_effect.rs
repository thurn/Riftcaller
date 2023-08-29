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

use serde::{Deserialize, Serialize};

use crate::game_actions::CardTarget;
use crate::primitives::{AbilityId, ActionCount, CardId, ManaValue, Side};

/// An arbitrary modification to the state of an ongoing game.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum GameEffect {
    /// Close the current prompt with no effect.
    Cancel,
    /// Put the indicated card into play
    PlayCard(CardId, CardTarget),
    /// Sacrifice the indicated permanent, moving it to its owner's discard
    /// pile.
    SacrificeCard(CardId),
    /// A player loses mana
    LoseMana(Side, ManaValue),
    /// A player loses action points
    LoseActions(Side, ActionCount),
    /// End the current raid in failure.
    EndRaid,
    /// Deal damage to the Champion
    TakeDamage(AbilityId, u32),
}