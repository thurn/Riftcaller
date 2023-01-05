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

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum TutorialMessageKey {
    DeckEditor,
}

/// Data model for the player's progress through the game's tutorial
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TutorialData {
    /// Which tutorial messages has the user seen?
    pub seen: HashSet<TutorialMessageKey>,
}

impl TutorialData {
    pub fn new() -> Self {
        Self::default()
    }
}
