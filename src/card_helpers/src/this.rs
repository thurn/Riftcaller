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

use game_data::delegates::{Delegate, EventDelegate, MutationFn};
use game_data::primitives::CardId;

/// A delegate which fires when a project card's triggered ability fires
pub fn is_triggered(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::ProjectTriggered(EventDelegate { requirement: crate::this_card, mutation })
}
