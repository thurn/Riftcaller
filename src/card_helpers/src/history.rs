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

use game_data::card_state::CardChoice;
use game_data::delegate_data::{RaidEvent, UsedWeapon};
use game_data::game_history::HistoryEvent;
use game_data::game_state::GameState;
use game_data::primitives::{AbilityId, CardId, HasAbilityId, RoomId};

/// Returns the record of game events which happened on the current
/// player's turn so far, not including the game action currently being
/// processed.
pub fn current_turn(game: &GameState) -> impl Iterator<Item = &HistoryEvent> + '_ {
    game.history.for_turn(game.info.turn)
}

/// Returns an iterator over cards which have been played in the current
/// player's turn so far.
///
/// Does not include the current event.
pub fn cards_played_this_turn(game: &GameState) -> impl Iterator<Item = CardId> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::PlayCard(id, _, _) = h {
            Some(*id)
        } else {
            None
        }
    })
}

/// Returns true if the `card_id` card was played this turn.
pub fn played_this_turn(game: &GameState, card_id: CardId) -> bool {
    cards_played_this_turn(game).any(|id| id == card_id)
}

/// Returns an iterator over abilities which have been activated in the current
/// player's turn so far.
///
/// Does not include the current event.
pub fn abilities_activated_this_turn(game: &GameState) -> impl Iterator<Item = AbilityId> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::ActivateAbility(id, _, _) = h {
            Some(*id)
        } else {
            None
        }
    })
}

/// Returns an iterator over rooms which have been raided in the current
/// player's turn so far.
pub fn rooms_raided_this_turn(game: &GameState) -> impl Iterator<Item = RoomId> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::RaidBegin(event) = h {
            Some(event.target)
        } else {
            None
        }
    })
}

/// Returns an iterator over rooms which have been successfully raided in the
/// current player's turn so far.
pub fn raid_accesses_this_turn(game: &GameState) -> impl Iterator<Item = RoomId> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::RaidSuccess(event) = h {
            Some(event.target)
        } else {
            None
        }
    })
}

/// Returns an iterator over minions which have been summoned in the current
/// player's turn so far.
pub fn minions_summoned_this_turn(
    game: &GameState,
) -> impl Iterator<Item = &RaidEvent<CardId>> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::MinionSummon(event) = h {
            Some(event)
        } else {
            None
        }
    })
}

/// Returns an iterator over minions which have been approached in the current
/// player's turn so far.
pub fn minions_approached_this_turn(
    game: &GameState,
) -> impl Iterator<Item = &RaidEvent<CardId>> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::MinionApproach(event) = h {
            Some(event)
        } else {
            None
        }
    })
}

/// Returns an iterator over minions which have been encountered in the current
/// player's turn so far.
pub fn minions_encountered_this_turn(
    game: &GameState,
) -> impl Iterator<Item = &RaidEvent<CardId>> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::MinionEncounter(event) = h {
            Some(event)
        } else {
            None
        }
    })
}

/// Returns an iterator over weapons which have been used in the current
/// player's turn so far.
pub fn weapons_used_this_turn(
    game: &GameState,
) -> impl Iterator<Item = &RaidEvent<UsedWeapon>> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::UseWeapon(event) = h {
            Some(event)
        } else {
            None
        }
    })
}

/// Returns an iterator over card choices which have been recorded in the
/// current player's turn so far for the given `ability_id`.
pub fn card_choices_this_turn(
    game: &GameState,
    has_ability_id: impl HasAbilityId,
) -> impl Iterator<Item = CardChoice> + '_ {
    let ability_id = has_ability_id.ability_id();
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::CardChoice(event) = h {
            if event.ability_id == ability_id {
                return Some(event.choice);
            }
        }
        None
    })
}
