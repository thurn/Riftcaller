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

use anyhow::Result;
use data::adventure::{AdventureChoiceScreen, AdventureState, TileEntity, TilePosition};
use data::adventure_action::AdventureAction;
use with_error::{fail, verify, WithError};

/// Handles an incoming [AdventureAction] and produces a client response.
pub fn handle_adventure_action(state: &mut AdventureState, action: &AdventureAction) -> Result<()> {
    match action {
        AdventureAction::AbandonAdventure => handle_abandon_adventure(state),
        AdventureAction::Explore(position) => handle_explore(state, *position),
        AdventureAction::InitiateDraft(position) => handle_initiate_draft(state, *position),
        AdventureAction::DraftCard(index) => handle_draft(state, *index),
    }
}

fn handle_abandon_adventure(state: &mut AdventureState) -> Result<()> {
    state.choice_screen = Some(AdventureChoiceScreen::AdventureOver);
    Ok(())
}

fn handle_explore(state: &mut AdventureState, position: TilePosition) -> Result<()> {
    verify_no_mandatory_choice(state)?;
    let (region, cost) = match state.tile_entity(position)? {
        TileEntity::Explore { region, cost } => (*region, *cost),
        _ => fail!("Expected explore entity"),
    };

    state.revealed_regions.insert(region);
    state.coins -= cost;
    state.tile_mut(position)?.entity = None;

    Ok(())
}

fn handle_initiate_draft(state: &mut AdventureState, position: TilePosition) -> Result<()> {
    verify_no_mandatory_choice(state)?;
    let cost = match state.tile_entity(position)? {
        TileEntity::Draft { cost, .. } => *cost,
        _ => fail!("Expected explore entity"),
    };

    state.coins -= cost;
    state.choice_screen = Some(AdventureChoiceScreen::Draft(position));
    state.tile_mut(position)?.entity = None;

    Ok(())
}

fn handle_draft(state: &mut AdventureState, index: usize) -> Result<()> {
    let Some(AdventureChoiceScreen::Draft(position)) = &state.choice_screen else {
        fail!("No active draft!");
    };

    let TileEntity::Draft { data, ..} = state.tile_entity(*position)? else {
        fail!("Invalid draft position");
    };

    let choice = data.choices.get(index).with_error(|| "Choice index out of bounds")?;
    let quantity = choice.quantity;
    state.collection.entry(choice.card).and_modify(|i| *i += quantity).or_insert(quantity);
    state.choice_screen = None;
    Ok(())
}

/// Other adventure actions cannot be take while a choice screen like 'draft a
/// card' is being displayed.
fn verify_no_mandatory_choice(state: &AdventureState) -> Result<()> {
    verify!(state.choice_screen.is_none(), "Mandatory choice screen is active!");
    Ok(())
}
