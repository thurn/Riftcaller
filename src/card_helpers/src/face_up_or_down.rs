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

use game_data::delegates::{Delegate, EventDelegate, MutationFn, Scope};
use game_data::game::GameState;
use game_data::primitives::TurnNumber;

/// RequirementFn that this delegate's card is currently in play, either face-up
/// or face-down.
pub fn in_play<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    game.card(scope.card_id()).position().in_play()
}

/// A delegate which triggers at dusk if a card is in play, either face-up or
/// face-down.
pub fn at_dusk(mutation: MutationFn<TurnNumber>) -> Delegate {
    Delegate::Dusk(EventDelegate { requirement: in_play, mutation })
}
