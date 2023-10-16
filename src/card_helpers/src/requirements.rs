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

use game_data::card_definition::TargetRequirement;
use game_data::delegate_data::DealtDamage;
#[allow(unused_imports)] // Used in Rustdocs
use game_data::delegate_data::{RequirementFn, Scope};
use game_data::game_actions::GamePrompt;
use game_data::game_history::{HistoryEvent, HistoryEventKind};
use game_data::game_state::GameState;
use game_data::primitives::{CardId, RaidId, RoomId};
use game_data::utils;
use rules::flags;

use crate::{face_down_in_play, history};

pub trait BaseRequirement {
    fn run(game: &GameState, scope: Scope) -> bool;
}

pub struct Always;
impl BaseRequirement for Always {
    fn run(_: &GameState, _: Scope) -> bool {
        true
    }
}

pub struct FaceUpInPlay;
impl BaseRequirement for FaceUpInPlay {
    fn run(game: &GameState, scope: Scope) -> bool {
        face_up_in_play(game, scope, &())
    }
}

pub struct FaceDownInPlay;
impl BaseRequirement for FaceDownInPlay {
    fn run(game: &GameState, scope: Scope) -> bool {
        face_down_in_play(game, scope, &())
    }
}

/// RequirementFn which always returns true
pub fn always<T>(_: &GameState, _: Scope, _: &T) -> bool {
    true
}

/// RequirementFn that this card is currently face up & in play
pub fn face_up_in_play<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    let card = game.card(scope.card_id());
    card.is_face_up() && card.position().in_play()
}

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

    if let Some(data) = game.state_machines.play_card {
        return data.card_id == *card_id && data.initiated_by.card_id() == Some(scope.card_id());
    }

    false
}

/// A [RequirementFn] which matches if there is a current raid which was
/// initiated by this card.
pub fn matching_raid<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    utils::is_true(|| {
        Some(game.raid.as_ref()?.initiated_by.ability_id()?.card_id == scope.card_id())
    })
}

/// A [RequirementFn] which matches if this weapon is face up in play and has
/// been used during the current raid
pub fn weapon_used_this_raid<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    let Some(raid_id) = game.raid.as_ref().map(|r| r.raid_id) else {
        return false;
    };

    if !face_up_in_play(game, scope, &()) {
        return false;
    }

    history::current_turn(game).any(|event| {
        matches!(event, HistoryEvent::UseWeapon(raid)
            if raid.raid_id == raid_id && raid.data.weapon_id == scope.card_id())
    })
}

/// A [RequirementFn] which matches if there have been no accesses on the
/// sanctum this turn.
pub fn no_sanctum_access<R: BaseRequirement>(game: &GameState, scope: Scope, _: &RaidId) -> bool {
    R::run(game, scope) && history::raid_accesses_this_turn(game).all(|r| r != RoomId::Sanctum)
}

/// A [RequirementFn] which matches if there have been no 'draw a card' actions
/// this turn
pub fn no_card_draw_actions<R: BaseRequirement>(
    game: &GameState,
    scope: Scope,
    _: &CardId,
) -> bool {
    R::run(game, scope)
        && history::current_turn(game).all(|e| e.kind() != HistoryEventKind::DrawCardAction)
}

/// A [RequirementFn] which matches if no damage has been dealt this turn.
pub fn no_damage_dealt<R: BaseRequirement>(
    game: &GameState,
    scope: Scope,
    _: &DealtDamage,
) -> bool {
    R::run(game, scope)
        && history::current_turn(game).all(|e| e.kind() != HistoryEventKind::DealDamage)
}

/// A `TargetRequirement` for a card which can target any room which is a valid
/// raid target
pub fn any_raid_target<T>() -> TargetRequirement<T> {
    TargetRequirement::TargetRoom(|game, _, room_id| flags::is_valid_raid_target(game, room_id))
}
