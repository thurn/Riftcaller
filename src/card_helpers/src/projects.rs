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

use game_data::card_definition::Ability;
use game_data::delegates::Scope;
use game_data::game::GameState;
use game_data::text::TextToken;

use crate::text;

/// RequirementFn that this delegate's card is currently in play, either face-up
/// or face-down.
pub fn either_face_in_play<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    game.card(scope.card_id()).position().in_play()
}

/// Ability to store a fixed amount of mana in a card when it is unveiled.
pub fn store_mana_on_unveil<const N: u32>() -> Ability {
    crate::simple_ability(
        crate::trigger_text(TextToken::Unveil, text![TextToken::StoreMana(N)]),
        crate::when_unveiled(|g, s, _| {
            crate::add_stored_mana(g, s.card_id(), N);
            Ok(())
        }),
    )
}
