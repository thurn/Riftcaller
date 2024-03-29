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

use core_data::game_primitives::{CardId, HasAbilityId, RoomId, Side};
use game_data::delegate_data::{AccessEvent, RaidEvent, UsedWeapon};
use game_data::game_state::{GameState, TurnData};
use game_data::history_data::{AbilityActivation, HistoryCounters, HistoryEvent};

/// Returns the record of game events which happened on the current
/// player's turn so far, not including the game action currently being
/// processed.
pub fn current_turn(game: &GameState) -> impl Iterator<Item = &HistoryEvent> + '_ {
    game.history.for_turn(game.info.turn)
}

/// Returns the record of game events which happened in the opponent's turn
/// immediately prior to the current turn.
///
/// Returns the empty iterator if invoked on the Covenant player's first turn.
pub fn last_turn(game: &GameState) -> impl Iterator<Item = &HistoryEvent> + '_ {
    let current = game.info.turn;
    game.history.for_turn(match current.side {
        Side::Covenant => {
            TurnData { side: Side::Riftcaller, turn_number: current.turn_number.saturating_sub(1) }
        }
        Side::Riftcaller => TurnData { side: Side::Covenant, turn_number: current.turn_number },
    })
}

/// Returns the [HistoryCounters] reference for the current game turn.
pub fn counters(game: &GameState, side: Side) -> &HistoryCounters {
    game.history.counters_for_turn(game.info.turn, side)
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

/// Returns an iterator over instances in which the provided ability has been
/// activated this turn, returning the selected [AbilityActivation]s.
///
/// Does not include the current event.
pub fn ability_activations_this_turn(
    game: &GameState,
    source: impl HasAbilityId,
) -> impl Iterator<Item = &AbilityActivation> + '_ {
    let ability_id = source.ability_id();
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::ActivateAbility(activation) = h {
            if activation.ability_id == ability_id {
                return Some(activation);
            }
        }

        None
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
pub fn rooms_accessed_this_turn(game: &GameState) -> impl Iterator<Item = RoomId> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::RaidSuccess(event) = h {
            Some(event.target)
        } else {
            None
        }
    })
}

/// Returns true if the indicated room has been accessed this turn.
pub fn accessed_this_turn(game: &GameState, room_id: RoomId) -> bool {
    rooms_accessed_this_turn(game).any(|r| r == room_id)
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

/// Returns an iterator over minions whose combat abilities have fired in the
/// current player's turn so far.
pub fn minion_combat_abilities_this_turn(
    game: &GameState,
) -> impl Iterator<Item = &RaidEvent<CardId>> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::MinionCombatAbility(event) = h {
            Some(event)
        } else {
            None
        }
    })
}

/// Returns an iterator over cards which have been razed during a raid access in
/// the current player's turn so far.
pub fn accessed_cards_razed_this_turn(
    game: &GameState,
) -> impl Iterator<Item = &AccessEvent<CardId>> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::RazeAccessedCard(event) = h {
            Some(event)
        } else {
            None
        }
    })
}

/// Returns an iterator over cards which have been scored during a raid access
/// in the current player's turn so far.
pub fn accessed_cards_scored_this_turn(
    game: &GameState,
) -> impl Iterator<Item = &AccessEvent<CardId>> + '_ {
    current_turn(game).filter_map(move |h| {
        if let HistoryEvent::ScoreAccessedCard(event) = h {
            Some(event)
        } else {
            None
        }
    })
}
