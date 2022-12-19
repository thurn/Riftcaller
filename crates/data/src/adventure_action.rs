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

use crate::adventure::TilePosition;
use crate::primitives::Side;
use crate::user_actions::UserAction;

/// Actions which can be taken for the 'adventure' game mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum AdventureAction {
    /// Initiate a new adventure, generating a new map and replacing any
    /// existing adventure.
    NewAdventure(Side),
    /// Take the associated action on the indicated map tile
    TileAction(TilePosition),
    /// Transition an adventure to the 'completed' state and display the
    /// adventure summary screen.
    AbandonAdventure,
    /// Remove a player's current adventure, i.e. to stop displaying the
    /// adventure summary screen.
    LeaveAdventure,
    /// Draft the card at the indicated index on the draft screen
    DraftCard(usize),
}

impl From<AdventureAction> for UserAction {
    fn from(a: AdventureAction) -> Self {
        UserAction::AdventureAction(a)
    }
}
