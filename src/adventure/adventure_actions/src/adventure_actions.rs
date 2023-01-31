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
    AdventureChoiceScreen, AdventureState, Coins, TileEntity, TilePosition,
};
use adventure_data::adventure_action::AdventureAction;
use anyhow::Result;
use with_error::{fail, verify};

/// Handles an incoming [AdventureAction] and produces a client response.
pub fn handle_adventure_action(state: &mut AdventureState, action: &AdventureAction) -> Result<()> {
    match action {
        AdventureAction::AbandonAdventure => handle_abandon_adventure(state),
        AdventureAction::Explore(position) => handle_explore(state, *position),
        AdventureAction::InitiateDraft(position) => handle_initiate_draft(state, *position),
        AdventureAction::DraftCard(index) => handle_draft(state, *index),
        AdventureAction::VisitShop(position) => handle_visit_shop(state, *position),
        AdventureAction::BuyCard(position, index) => handle_buy_card(state, *position, *index),
    }
}

fn handle_abandon_adventure(state: &mut AdventureState) -> Result<()> {
    state.choice_screen = Some(AdventureChoiceScreen::AdventureOver);
    Ok(())
}

fn handle_explore(state: &mut AdventureState, position: TilePosition) -> Result<()> {
    verify_no_mandatory_choice(state)?;
    verify_revealed(state, position)?;

    let (region, cost) = match state.tile_entity(position)? {
        TileEntity::Explore { region, cost } => (*region, *cost),
        _ => fail!("Expected explore entity"),
    };

    spend_coins(state, cost)?;
    state.revealed_regions.insert(region);
    state.tile_mut(position)?.entity = None;

    Ok(())
}

fn handle_initiate_draft(state: &mut AdventureState, position: TilePosition) -> Result<()> {
    verify_no_mandatory_choice(state)?;
    verify_revealed(state, position)?;

    let cost = match state.tile_entity(position)? {
        TileEntity::Draft { cost, .. } => *cost,
        _ => fail!("Expected explore entity"),
    };

    spend_coins(state, cost)?;
    state.choice_screen = Some(AdventureChoiceScreen::Draft(position));

    Ok(())
}

fn handle_draft(state: &mut AdventureState, index: usize) -> Result<()> {
    let Some(AdventureChoiceScreen::Draft(position)) = &state.choice_screen else {
        fail!("No active draft!");
    };

    let TileEntity::Draft { data, ..} = state.tile_entity(*position)? else {
        fail!("Invalid draft position");
    };

    verify!(index < data.choices.len(), "Index out of bounds!");
    let choice = data.choices[index];

    state
        .collection
        .entry(choice.card)
        .and_modify(|i| *i += choice.quantity)
        .or_insert(choice.quantity);
    state.tile_mut(*position)?.entity = None;
    state.choice_screen = None;
    Ok(())
}

fn handle_visit_shop(state: &mut AdventureState, position: TilePosition) -> Result<()> {
    let TileEntity::Shop { data } = state.tile_entity_mut(position)? else {
        fail!("Expected shop entity")
    };

    data.visited = true;
    Ok(())
}

fn handle_buy_card(state: &mut AdventureState, position: TilePosition, index: usize) -> Result<()> {
    let TileEntity::Shop { data } = state.tile_entity_mut(position)? else {
        fail!("Expected shop entity")
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

/// Raise an error if the given [TilePosition] has not yet been explored
fn verify_revealed(state: &AdventureState, position: TilePosition) -> Result<()> {
    verify!(
        state.revealed_regions.contains(&state.tile(position)?.region_id),
        "Given tile position has not been revealed"
    );
    Ok(())
}

/// Other adventure actions cannot be take while a choice screen like 'draft a
/// card' is being displayed.
fn verify_no_mandatory_choice(state: &AdventureState) -> Result<()> {
    verify!(state.choice_screen.is_none(), "Mandatory choice screen is active!");
    Ok(())
}
