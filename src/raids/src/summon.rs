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
use game_data::game_actions::{PromptAction, SummonAction};
use game_data::primitives::Side;
use rules::mutations;
use with_error::{fail, verify};

use crate::defenders;
use crate::mutations::SummonMinion;
use crate::traits::{RaidDisplayState, RaidPhaseImpl};

/// The primary combat phase of a raid, in which the Champion may use weapon
/// abilities to attempt to defeat an active Overlord minion.
#[derive(Debug, Clone, Copy)]
pub struct SummonPhase {}

impl RaidPhaseImpl for SummonPhase {
    type Action = SummonAction;

    fn unwrap(action: PromptAction) -> Result<Self::Action> {
        match action {
            PromptAction::SummonAction(action) => Ok(action),
            _ => fail!("Expected SummonAction"),
        }
    }

    fn wrap(action: Self::Action) -> Result<PromptAction> {
        Ok(PromptAction::SummonAction(action))
    }

    fn enter(self, _game: &mut GameState) -> Result<Option<InternalRaidPhase>> {
        Ok(None)
    }

    fn actions(self, game: &GameState) -> Result<Vec<Self::Action>> {
        Ok(vec![SummonAction::SummonMinion(game.raid_defender()?), SummonAction::DoNotSummmon])
    }

    fn handle_action(
        self,
        game: &mut GameState,
        action: Self::Action,
    ) -> Result<Option<InternalRaidPhase>> {
        match action {
            SummonAction::SummonMinion(minion_id) => {
                verify!(minion_id == game.raid_defender()?, "Invalid minion id");
                verify!(defenders::can_summon_defender(game, minion_id)?, "Cannot summon minion");
                mutations::summon_minion(game, minion_id, SummonMinion::PayCosts)?;

                if game.info.raid.is_none() {
                    // Minion ability may have ended raid
                    Ok(None)
                } else {
                    Ok(Some(InternalRaidPhase::Encounter))
                }
            }
            SummonAction::DoNotSummmon => defenders::advance_to_next_encounter(game),
        }
    }

    fn active_side(self) -> Side {
        Side::Overlord
    }

    fn display_state(self, game: &GameState) -> Result<RaidDisplayState> {
        defenders::defender_list_display_state(game)
    }
}
