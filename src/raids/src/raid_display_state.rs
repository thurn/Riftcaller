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

use game_data::game::GameState;
use game_data::raid_data::{RaidData, RaidDisplayState, RaidStatus};
use rules::queries;

/// Generates the current [RaidDisplayState] for the game, which controls how
/// cards are positioned during a raid.
pub fn build(game: &GameState) -> RaidDisplayState {
    let Some(raid) = &game.raid else {
        return RaidDisplayState::None;
    };

    match queries::raid_status(raid) {
        RaidStatus::Begin => RaidDisplayState::None,
        RaidStatus::Summon | RaidStatus::Encounter => defenders(game, raid),
        RaidStatus::ApproachRoom => RaidDisplayState::Defenders(vec![]),
        RaidStatus::Access => RaidDisplayState::Access,
    }
}

fn defenders(game: &GameState, raid: &RaidData) -> RaidDisplayState {
    let defenders = game.defender_list(raid.target);
    RaidDisplayState::Defenders(defenders[0..=raid.encounter].to_vec())
}
