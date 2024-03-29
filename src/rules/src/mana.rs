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

//! Manages mana, especially logic around "spend this mana only on X"
//! restrictions.

use std::cmp;

use anyhow::Result;
use core_data::game_primitives::{AbilityId, CardId, InitiatedBy, ManaValue, RaidId, RoomId, Side};
use dispatcher::dispatch;
use game_data::delegate_data::{ManaLostToOpponentAbility, ManaLostToOpponentAbilityEvent};
use game_data::game_state::GameState;
use game_data::utils;
use tracing::debug;
use with_error::verify;

/// Identifies possible reasons why a player's mana value would need to be
/// queried or spent.
#[derive(Debug, Clone, Copy)]
pub enum ManaPurpose {
    BaseMana,
    BonusForDisplay,
    PayForCard(CardId),
    RazeCard(CardId),
    UseWeapon(CardId),
    ActivateAbility(AbilityId),
    ProgressRoom(RoomId),
    PayForTriggeredAbility,
    CombatAbility,
    RemoveCurse,
    DispelEvocation,
    AdditionalActionCost,
    AllSources,
}

/// Queries the amount of mana available for the `side` player when used for the
/// given [ManaPurpose].
///
/// Certain card effects may grant mana conditionally for a given purpose.
pub fn get(game: &GameState, side: Side, purpose: ManaPurpose) -> ManaValue {
    let base_mana = game.player(side).mana_state.base_mana;
    let mut result = game.player(side).mana_state.base_mana;
    match (&game.raid, &game.player(side).mana_state.raid_mana) {
        (Some(raid_data), Some((raid_id, raid_mana))) if raid_data.raid_id == *raid_id => {
            result += raid_mana;
        }
        _ => {}
    }

    match purpose {
        ManaPurpose::BaseMana => base_mana,
        ManaPurpose::BonusForDisplay => result - base_mana,
        _ => result,
    }
}

/// Spends mana for the `side` player for the given [ManaPurpose].
///
/// An effort is made to spend "more specific" mana first, i.e. mana which can
/// only be used for a certain type of action is preferred, then raid-specific
/// mana, then general mana.
///
/// Returns an error if insufficient mana is available.
pub fn spend(
    game: &mut GameState,
    side: Side,
    initiated_by: InitiatedBy,
    purpose: ManaPurpose,
    amount: ManaValue,
) -> Result<()> {
    debug!(?amount, ?side, "Spending mana");
    verify!(get(game, side, purpose) >= amount);
    let mut to_spend = amount;

    if let Some(current_raid_id) = game.raid.as_ref().map(|r| r.raid_id) {
        if let Some((raid_id, mana)) = &mut game.player_mut(side).mana_state.raid_mana {
            if *raid_id == current_raid_id {
                // Sure wish Rust would stabilize if_chains already...
                to_spend = try_spend(mana, to_spend);
            }
        }
    }

    game.player_mut(side).mana_state.base_mana -= to_spend;

    match initiated_by {
        InitiatedBy::Ability(ability_id) if ability_id.side() != side => {
            dispatch::invoke_event(
                game,
                ManaLostToOpponentAbilityEvent(&ManaLostToOpponentAbility { side, amount }),
            )?;
        }
        _ => {}
    }
    Ok(())
}

/// Causes a player to lose up to a given amount of mana.
pub fn lose_upto(
    game: &mut GameState,
    side: Side,
    initiated_by: InitiatedBy,
    purpose: ManaPurpose,
    amount: ManaValue,
) -> Result<()> {
    debug!(?amount, ?side, "Losing up to 'amount' mana");
    spend(game, side, initiated_by, purpose, cmp::min(get(game, side, purpose), amount))
}

/// Adds the specified amount of base mana (no restrictions on use) for the
/// `side` player.
pub fn gain(game: &mut GameState, side: Side, amount: ManaValue) {
    debug!(?amount, ?side, "Gaining mana");
    game.player_mut(side).mana_state.base_mana += amount
}

/// Sets an initial amount of base mana for the `side` player.
pub fn set_initial(game: &mut GameState, side: Side, amount: ManaValue) {
    debug!(?amount, ?side, "Setting mana");
    game.player_mut(side).mana_state.base_mana = amount;
}

/// Adds mana for the `side` player which can only be used during the specified
/// `raid_id` raid.
pub fn add_raid_specific_mana(
    game: &mut GameState,
    side: Side,
    raid_id: RaidId,
    amount: ManaValue,
) {
    debug!(?amount, ?side, ?raid_id, "Adding raid-specific mana");
    utils::add_matching(&mut game.player_mut(side).mana_state.raid_mana, raid_id, amount);
}

fn try_spend(source: &mut ManaValue, amount: ManaValue) -> ManaValue {
    if *source >= amount {
        *source -= amount;
        0
    } else {
        let result = amount - *source;
        *source = 0;
        result
    }
}
