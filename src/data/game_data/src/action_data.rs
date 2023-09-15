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

use serde::{Deserialize, Serialize};

use crate::card_state::CardPosition;
use crate::game_actions::CardTarget;
use crate::primitives::CardId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PlayCardStep {
    Begin,
    CheckLimits,
    MoveToPlayedPosition,
    PayActionPoints,
    ClearBrowser,
    PayManaCost,
    PayCustomCost,
    TurnFaceUp,
    MoveToTargetPosition,
    Finish,
}

/// Data related to an ongoing action to play a card.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayCardData {
    /// Card being played
    pub card_id: CardId,
    /// Where should this card be moved if the play card action is
    /// aborted/finished at the current step?
    pub original_position: CardPosition,
    /// Room being targeted, if any
    pub target: CardTarget,
    /// Current state machine state
    pub step: PlayCardStep,
}

/// Data related to a game action which is currently in the process of
/// resolving. Game actions are handled via a resumable state machine in order
/// to allow interruptions in the resolution process when a player is required
/// to make a prompt decision.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActionData {
    PlayCard(PlayCardData),
}
