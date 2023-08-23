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
use game_data::game::{GameState, InternalRaidPhase};
use game_data::game_actions::{ApproachRoomAction, PromptAction};
use game_data::primitives::Side;
use rules::flags;
use with_error::fail;

use crate::traits::{RaidDisplayState, RaidPhaseImpl};

/// The Champion has bypassed all of the defenders for this room and the
/// Overlord has one final opportunity to take actions before cards are
/// accessed.
#[derive(Debug, Clone, Copy)]
pub struct ApproachRoomPhase {}

impl RaidPhaseImpl for ApproachRoomPhase {
    type Action = ApproachRoomAction;

    fn unwrap(action: PromptAction) -> Result<Self::Action> {
        match action {
            PromptAction::ApproachRoomAction(action) => Ok(action),
            _ => fail!("Expected ApproachRoomAction"),
        }
    }

    fn wrap(action: Self::Action) -> Result<PromptAction> {
        Ok(PromptAction::ApproachRoomAction(action))
    }

    fn enter(self, g: &mut GameState) -> Result<Option<InternalRaidPhase>> {
        if g.occupants_in_all_rooms()
            .any(|c| flags::can_take_unveil_card_action(g, Side::Overlord, c.id))
        {
            Ok(None)
        } else {
            Ok(Some(InternalRaidPhase::Access))
        }
    }

    fn actions(self, _: &GameState) -> Result<Vec<Self::Action>> {
        Ok(vec![ApproachRoomAction::Proceed])
    }

    fn handle_action(
        self,
        _: &mut GameState,
        _: Self::Action,
    ) -> Result<Option<InternalRaidPhase>> {
        Ok(Some(InternalRaidPhase::Access))
    }

    fn display_state(self, _: &GameState) -> Result<RaidDisplayState> {
        Ok(RaidDisplayState::Defenders(vec![]))
    }
}
