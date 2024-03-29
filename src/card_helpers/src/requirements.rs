// Copyright © Riftcaller 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use card_definition_data::cards::CardDefinitionExt;
use core_data::game_primitives::{HasCardId, RaidId, RoomId, Side};
use game_data::card_configuration::TargetRequirement;
use game_data::card_state::CardPosition;
use game_data::delegate_data::DealtDamage;
#[allow(unused_imports)] // Used in Rustdocs
use game_data::delegate_data::{RequirementFn, Scope};
use game_data::game_state::GameState;
use game_data::history_data::{HistoryEvent, HistoryEventKind};
use game_data::prompt_data::GamePrompt;
use game_data::utils;
use rules::{flags, play_card, prompts};

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

/// RequirementFn that matches if this card is currently in hand
pub fn in_hand<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    game.card(scope.card_id()).position() == CardPosition::Hand(scope.side())
}

/// A [RequirementFn] which matches while the `card_id` card is either:
///
///   1) Displayed in a PlayCardBrowser initiated by the this card, or
///   2) Currently being played as part of a 'play card' action initiated by
///      this card.
pub fn matching_play_browser(game: &GameState, scope: Scope, id: &impl HasCardId) -> bool {
    let card_id = id.card_id();
    if let Some(GamePrompt::PlayCardBrowser(browser)) = prompts::current(game, card_id.side) {
        if browser.cards.contains(&card_id) && browser.initiated_by.card_id == scope.card_id() {
            return true;
        }
    }

    play_card::currently_being_played_by(game, card_id, scope)
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

/// A [RequirementFn] which matches if this minion's combat ability has fired
/// during the current turn.
pub fn combat_ability_fired_this_turn<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    history::minion_combat_abilities_this_turn(game).any(|card_id| card_id.data == scope.card_id())
}

/// A [RequirementFn] which matches if there have been no accesses on the
/// sanctum this turn.
pub fn no_sanctum_access<R: BaseRequirement>(game: &GameState, scope: Scope, _: &RaidId) -> bool {
    R::run(game, scope) && history::rooms_accessed_this_turn(game).all(|r| r != RoomId::Sanctum)
}

/// A [RequirementFn] which matches if there have been no 'draw a card' actions
/// this turn
pub fn no_card_draw_actions<R: BaseRequirement>(game: &GameState, scope: Scope, _: &Side) -> bool {
    R::run(game, scope)
        && history::current_turn(game).all(|e| e.kind() != HistoryEventKind::DrawCardAction)
}

/// A [RequirementFn] which matches if no damage has been dealt this turn.
pub fn no_damage_dealt<R: BaseRequirement>(
    game: &GameState,
    scope: Scope,
    _: &DealtDamage,
) -> bool {
    R::run(game, scope) && history::counters(game, Side::Riftcaller).damage_received == 0
}

/// A [RequirementFn] which matches if the ability associated with the
/// current [Scope] has been activated this this turn.
pub fn ability_activated_this_turn<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    history::ability_activations_this_turn(game, scope.ability_id()).next().is_some()
}

/// A [RequirementFn] which matches if the indicated `card_id` was targeted by
/// the parent card for the current turn via
/// `CustomCardState::TargetCardForTurn`.
pub fn card_targeted_for_this_turn(
    game: &GameState,
    source: impl HasCardId,
    card_id: &impl HasCardId,
) -> bool {
    game.card(source)
        .custom_state
        .targets_for_turn(game.info.turn)
        .any(|id| id == card_id.card_id())
}

/// A [RequirementFn] which matches if the indicated `card_id` was targeted by
/// the parent card for the current turn or in the immediate prior turn via
/// `CustomCardState::TargetCardForTurn`.
pub fn card_targeted_for_this_turn_cycle(
    game: &GameState,
    source: impl HasCardId,
    card_id: &impl HasCardId,
) -> bool {
    game.card(source)
        .custom_state
        .targets_for_turn_cycle(game.info.turn)
        .any(|id| id == card_id.card_id())
}

/// A [RequirementFn] which matches if the `source` card has been marked as
/// active during the current raid encounter.
pub fn active_this_encounter<T>(game: &GameState, source: impl HasCardId, _: &T) -> bool {
    utils::is_true(|| {
        Some(
            game.card(source.card_id())
                .custom_state
                .is_active_for_encounter(game.raid.as_ref()?.minion_encounter_id?),
        )
    })
}

/// A `TargetRequirement` for a card which can target any room
pub fn any_room<T>() -> TargetRequirement<T> {
    TargetRequirement::TargetRoom(|_, _, _| true)
}

/// A `TargetRequirement` for a card which can target any room which is a valid
/// raid target
pub fn any_raid_target<T>() -> TargetRequirement<T> {
    TargetRequirement::TargetRoom(|game, _, room_id| flags::is_valid_raid_target(game, room_id))
}

/// A `TargetRequirement` for a card which can target any outer room which is a
/// valid raid target
pub fn any_outer_room_raid_target<T>() -> TargetRequirement<T> {
    TargetRequirement::TargetRoom(|game, _, room_id| {
        room_id.is_outer_room() && flags::is_valid_raid_target(game, room_id)
    })
}

/// A [TargetRequirement] targeting rooms with occupants.
pub fn occupied_room<T>() -> TargetRequirement<T> {
    TargetRequirement::TargetRoom(|game, _, room_id| game.occupants(room_id).next().is_some())
}

/// A [TargetRequirement] targeting rooms with defenders.
pub fn defended_room<T>() -> TargetRequirement<T> {
    TargetRequirement::TargetRoom(|game, _, room_id| {
        game.defenders_unordered(room_id).next().is_some()
    })
}

/// A [TargetRequirement] targeting rooms with defenders or occupants.
pub fn defended_or_occupied<T>() -> TargetRequirement<T> {
    TargetRequirement::TargetRoom(|game, _, room_id| {
        game.defenders_and_occupants(room_id).next().is_some()
    })
}

/// A [TargetRequirement] targeting rooms with Infernal defenders.
pub fn any_room_with_infernal_defenders<T>() -> TargetRequirement<T> {
    TargetRequirement::TargetRoom(|game, _, room_id| {
        game.defenders_unordered(room_id).any(|card| game.card(card.id).definition().is_infernal())
    })
}
