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

//! Implements game rules for the 'adventure' deckbuilding/drafting game mode

use adventure_data::adventure::{
    AdventureOutcome, AdventureState, Coins, TileEntity, TilePosition,
};
use adventure_data::adventure_action::AdventureAction;
use anyhow::Result;
use game_data::primitives::CardType;
use with_error::{fail, verify};

/// Handles an incoming [AdventureAction] and produces a client response.
pub fn handle_adventure_action(state: &mut AdventureState, action: &AdventureAction) -> Result<()> {
    match action {
        AdventureAction::AbandonAdventure => handle_abandon_adventure(state),
        AdventureAction::VisitTileEntity(position) => handle_visit_tile(state, *position),
        AdventureAction::EndVisit => handle_end_visit(state),
        AdventureAction::DraftCard(index) => handle_draft(state, *index),
        AdventureAction::BuyCard(index) => handle_buy_card(state, *index),
    }
}

fn handle_abandon_adventure(state: &mut AdventureState) -> Result<()> {
    state.outcome = Some(AdventureOutcome::Defeat);
    Ok(())
}

fn handle_visit_tile(state: &mut AdventureState, position: TilePosition) -> Result<()> {
    verify!(
        state.revealed_regions.contains(&state.tile(position)?.region_id),
        "Given tile position has not been revealed"
    );
    state.tile_mut(position)?.visited = true;
    state.visiting_position = Some(position);
    Ok(())
}

fn handle_end_visit(state: &mut AdventureState) -> Result<()> {
    verify!(is_blocking_screen(state) != Some(true), "Cannot end visit on this screen");
    state.visiting_position = None;
    Ok(())
}

/// Returns Some(true) if the player cannot end a visit on the current screen
/// without taking some other game action.
fn is_blocking_screen(state: &mut AdventureState) -> Option<bool> {
    let position = state.visiting_position?;
    match state.tiles.get(&position)?.entity.as_ref()? {
        TileEntity::Draft(_) => Some(true),
        _ => None,
    }
}

fn handle_draft(state: &mut AdventureState, index: usize) -> Result<()> {
    let TileEntity::Draft(data) = state.visiting_tile_mut()? else {
        fail!("Expected active draft screen");
    };

    verify!(index < data.choices.len(), "Index out of bounds!");
    let choice = data.choices[index];

    let definition = rules::get(choice.card);
    if definition.card_type == CardType::Sigil {
        if !state.deck.schools.contains(&definition.school) {
            state.deck.schools.push(definition.school);
        }
        state.deck.sigils.push(definition.name);
    } else {
        state
            .collection
            .entry(choice.card)
            .and_modify(|i| *i += choice.quantity)
            .or_insert(choice.quantity);
    }

    state.clear_visited_tile()?;
    Ok(())
}

fn handle_buy_card(state: &mut AdventureState, index: usize) -> Result<()> {
    let TileEntity::Shop(data) = state.visiting_tile_mut()? else {
        fail!("Expected active shop screen");
    };

    verify!(index < data.choices.len(), "Index out of bounds!");
    let choice = data.choices[index];
    verify!(!choice.sold, "Item already sold!");
    data.choices[index].sold = true;

    state
        .collection
        .entry(choice.card)
        .and_modify(|i| *i += choice.quantity)
        .or_insert(choice.quantity);
    spend_coins(state, choice.cost)?;

    Ok(())
}

fn spend_coins(state: &mut AdventureState, coins: Coins) -> Result<()> {
    verify!(state.coins >= coins, "Insufficient coins available");
    state.coins -= coins;
    Ok(())
}
