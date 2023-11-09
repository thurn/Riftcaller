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
use game_data::animation_tracker::InitiatedBy;
use game_data::delegate_data::{Delegate, QueryDelegate, RequirementFn, Scope};
use game_data::game_actions::CardTarget;
use game_data::game_state::GameState;
use game_data::primitives::{CardId, RaidId};
use game_data::raid_data::{RaidState, RaidStatus};

/// Starts a new raid from a card ability associated with the provided [Scope]
/// and [CardTarget] room.
pub fn initiate(game: &mut GameState, scope: Scope, target: CardTarget) -> Result<()> {
    raid_state::initiate_with_callback(
        game,
        target.room_id()?,
        InitiatedBy::Ability(scope.ability_id()),
        |_, _| {},
    )
}

pub fn add_vault_access<const N: u32>(requirement: RequirementFn<RaidId>) -> Delegate {
    Delegate::VaultAccessCount(QueryDelegate {
        requirement,
        transformation: |_, _, _, current| current + N,
    })
}

pub fn add_sanctum_access<const N: u32>(requirement: RequirementFn<RaidId>) -> Delegate {
    Delegate::SanctumAccessCount(QueryDelegate {
        requirement,
        transformation: |_, _, _, current| current + N,
    })
}

/// Returns the minion currently being encountered if there is an active raid
/// encounter prompt in this game.
pub fn active_encounter_prompt(game: &GameState) -> Option<CardId> {
    game.raid.as_ref().and_then(|raid| match &raid.state {
        RaidState::Prompt(p) if p.status == RaidStatus::Encounter => game.current_raid_defender(),
        _ => None,
    })
}
