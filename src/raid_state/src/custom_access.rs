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

use anyhow::Result;
use core_data::game_primitives::{CardId, InitiatedBy, RaidId, RoomId};
use game_data::delegate_data::CustomAccessEndEvent;
use game_data::game_state::GameState;
use game_data::raid_data::{RaidData, RaidState, RaidStep};
use rules::{dispatch, flags};
use with_error::verify;

/// Initiates a "Custom Access" raid.
///
/// A custom access raid is a raid that runs through the "access" phase for a
/// room on a given set of cards, which might be described in card text with
/// phrasing like "access the top 5 cards of the vault".
///
/// Custom access raids fire events and add history entries directly related to
/// scoring and accessing cards:
///
///  - CardAccessEvent
///  - ChampionScoreCardEvent
///  - ScoreCardEvent
///  - RazeCardEvent
///  - HistoryEvent::ScoreAccessedCard
///  - HistoryEvent::RazeAccessedCard
///
/// They do *not* fire events or add history entries related to overall raid
/// state, such as:
///
///  - RaidAccessStartEvent
///  - RaidAccessSelectedEvent
///  - RaidAccessEndEvent
///  - RaidSuccessEvent
///  - RaidEndEvent
///  - HistoryEvent::RaidSuccess
///
/// "CustomAccessEndEvent" is fired when a custom access is completed.
///
/// The game *is* considered to have an active raid with the given target during
/// a custom access and any "during a raid" conditions are considered true.
/// However, the raid does not count as a "room access" for effects like "if you
/// accessed the vault this turn".
pub fn initiate(
    game: &mut GameState,
    target: RoomId,
    initiated_by: InitiatedBy,
    accessed: Vec<CardId>,
) -> Result<()> {
    verify!(!flags::raid_active(game), "Raid is already active");

    game.raid = Some(RaidData {
        target,
        initiated_by,
        raid_id: RaidId(game.info.next_event_id()),
        state: RaidState::Step(RaidStep::RevealAccessedCards),
        encounter: game.defenders_unordered(target).count(),
        minion_encounter_id: None,
        room_access_id: None,
        accessed,
        jump_request: None,
        is_card_access_prevented: false,
        is_custom_access: true,
    });

    crate::run(game, None)
}

/// Ends an ongoing custom access raid.
pub fn end(game: &mut GameState, initiated_by: InitiatedBy) -> Result<()> {
    dispatch::invoke_event(game, CustomAccessEndEvent(initiated_by))?;
    game.raid = None;
    Ok(())
}
