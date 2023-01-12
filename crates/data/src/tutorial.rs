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
    skip_all: bool,
    seen: HashSet<TutorialMessageKey>,
}

impl TutorialData {
    /// New default instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Avoid displaying any tutorial messages
    pub fn skip_all(mut self, skip_all: bool) -> Self {
        self.skip_all = skip_all;
        self
    }

    /// Returns true if the user has seen the tutorial message with the given
    /// key.
    pub fn has_seen(&self, key: TutorialMessageKey) -> bool {
        if self.skip_all {
            true
        } else {
            self.seen.contains(&key)
        }
    }

    /// Record that the user has seen the tutorial message with the given key
    pub fn mark_seen(&mut self, key: TutorialMessageKey) -> bool {
        self.seen.insert(key)
    }
}
