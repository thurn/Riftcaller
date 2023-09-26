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

#[allow(unused_imports)] // Used in Rustdocs
use game_data::delegates::{RequirementFn, Scope};
use game_data::game::GameState;
use game_data::game_actions::GamePrompt;
use game_data::game_history::HistoryEvent;
use game_data::primitives::CardId;
use game_data::utils;

/// A [RequirementFn] which matches while the `card_id` card is either:
///
///   1) Displayed in a PlayCardBrowser initiated by the this card, or
///   2) Currently being played as part of a 'play card' action initiated by
///      this card.
pub fn matching_play_browser(game: &GameState, scope: Scope, card_id: &CardId) -> bool {
    if let Some(GamePrompt::PlayCardBrowser(browser)) =
        game.player(card_id.side).prompt_queue.get(0)
    {
        if browser.cards.contains(card_id) && browser.initiated_by.card_id == scope.card_id() {
            return true;
        }
    }

    if let Some(HistoryEvent::PlayCard(id, _, initiated_by)) = game.history.current_event() {
        return id == card_id && initiated_by.card_id() == Some(scope.card_id());
    }

    false
}

/// A RequirementFn which checks if there is a current raid which was initiated
/// by this card.
pub fn matching_raid<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    utils::is_true(|| {
        Some(game.raid.as_ref()?.initiated_by.ability_id()?.card_id == scope.card_id())
    })
}
