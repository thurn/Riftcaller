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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::game::TurnData;
use crate::game_actions::CardTarget;
use crate::game_updates::InitiatedBy;
use crate::primitives::{AbilityId, CardId, ProgressValue, RoomId};

/// Records a single event which happened during this game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryEvent {
    /// Mana was gained via the standard game action
    GainManaAction,
    /// A card was drawn via the standard game action
    DrawCardAction(CardId),
    /// A card was played, either via the standard game action or initiated by
    /// [an ability of another card.
    PlayCard(CardId, CardTarget, InitiatedBy),
    /// A card ability was activated
    ActivateAbility(AbilityId, CardTarget),
    /// A face-down card has been unveiled.
    UnveilCard(CardId),
    /// A raid was started, either via a card effect or the standard game action
    RaidBegin(RoomId, InitiatedBy),
    /// A raid ended in success.
    RaidSuccess(RoomId),
    /// A raid ended in failure.
    RaidFailure(RoomId),
    /// A card was progressed some number of times, either via a card effect or
    /// the standard game action
    CardProgress(RoomId, ProgressValue, InitiatedBy),
}

/// History of events which have happened during this game. This contains both
/// the ongoing 'current event' as well as a sequence of past events for each
/// turn.
#[serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameHistory {
    current: Option<(HistoryEvent, TurnData)>,
    #[serde_as(as = "Vec<(_, _)>")]
    entries: HashMap<TurnData, Vec<HistoryEvent>>,
}

impl GameHistory {
    /// Returns the game event currently being processed, if any
    pub fn current_event(&self) -> Option<&HistoryEvent> {
        if let Some((event, _)) = &self.current {
            Some(event)
        } else {
            None
        }
    }

    /// Returns history events in the provided turn, *not* counting the
    /// [Self::current_event] value.
    pub fn for_turn(&self, turn: TurnData) -> impl Iterator<Item = &HistoryEvent> {
        self.entries.get(&turn).into_iter().flatten()
    }

    /// Starts a new history entry, setting it as the current event. Events do
    /// not appear in the [Self::for_turn] history until they are finalized by
    /// setting a new current event or calling
    /// [Self::finish_current_event_if_needed].
    ///
    /// Adds the previously current event to general game history if needed.
    pub fn add_event(&mut self, turn: TurnData, event: HistoryEvent) {
        self.finish_current_event_if_needed();
        self.current = Some((event, turn));
    }

    /// Clears the [Self::current_event] value and writes it to the general game
    /// history, if one is present.
    pub fn finish_current_event_if_needed(&mut self) {
        if let Some((event, turn)) = self.current.take() {
            self.entries.entry(turn).or_default().push(event);
        }
    }
}
