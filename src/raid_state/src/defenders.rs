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
use game_data::game_state::GameState;
use game_data::primitives::{CardId, Side};
use game_data::raid_data::{RaidInfo, RaidStep};
use rules::mana::ManaPurpose;
use rules::{mana, queries, CardDefinitionExt};
use with_error::WithError;

/// Returns true if the raid `defender_id` is currently face down and could be
/// turned face up automatically by paying its mana cost.
///
/// Returns an error if there is no active raid or if this is an invalid
/// defender.
pub fn can_summon_defender(game: &GameState, defender_id: CardId) -> bool {
    let mut can_summon = game.card(defender_id).is_face_down();

    if let Some(cost) = queries::mana_cost(game, defender_id) {
        can_summon &= cost <= mana::get(game, Side::Overlord, ManaPurpose::PayForCard(defender_id))
    }

    if let Some(custom_cost) = &game.card(defender_id).definition().cost.custom_cost {
        can_summon &= (custom_cost.can_pay)(game, defender_id);
    }

    can_summon
}

/// Mutates the provided game to update the current raid encounter to the next
/// available encounter number, if one is available. Returns the next
/// [RaidStep] which should be entered, based on whether a suitable
/// encounter was found.
pub fn next_encounter(game: &mut GameState, info: RaidInfo) -> Result<RaidStep> {
    Ok(if let Some(encounter) = next_defender(game, info, info.encounter) {
        game.raid_mut()?.encounter = encounter;
        let defender = game.current_raid_defender().with_error(|| "No defender found")?;
        if game.card(defender).is_face_down() {
            RaidStep::PopulateSummonPrompt(defender)
        } else {
            RaidStep::EncounterMinion(defender)
        }
    } else {
        RaidStep::PopulateApproachPrompt
    })
}

/// Searches for the next defender to encounter during an ongoing raid with a
/// position less than the provided index (or any index if not provided). If an
/// eligible defender is available with position < `less_than`, its index is
/// returned.
///
/// An 'eligible' defender is either one which is face up, or one which *can* be
/// turned face up by paying its costs.
fn next_defender(game: &GameState, info: RaidInfo, less_than: usize) -> Option<usize> {
    let target = info.target;
    let defenders = game.defender_list(target);
    let found = defenders.iter().enumerate().rev().find(|(index, card_id)| {
        let in_range = *index < less_than;
        in_range && (game.card(**card_id).is_face_up() || can_summon_defender(game, **card_id))
    });

    found.map(|(index, _)| index)
}
