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
use crate::user_actions::UserAction;

/// Actions which can be taken for the 'adventure' game mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum AdventureAction {
    /// Transition an adventure to the 'completed' state and display the
    /// adventure summary screen. Can be followed by
    /// `UserAction::LeaveAdventure` to completely exit the adventure.
    AbandonAdventure,
    /// Pay costs & explore more map tiles from the given position
    Explore(TilePosition),
    /// Start a new draft & pay costs for the given position
    InitiateDraft(TilePosition),
    /// Draft the card at the indicated index on the draft screen
    DraftCard(usize),
    /// Draft the purchase at the indicated index on a shop screen
    BuyCard(TilePosition, usize),
}

impl From<AdventureAction> for UserAction {
    fn from(a: AdventureAction) -> Self {
        UserAction::AdventureAction(a)
    }
}
