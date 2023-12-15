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

//! Implements game rules for the 'adventure' deckbuilding/drafting game mode

use adventure_data::adventure::{AdventureScreen, AdventureState};
use adventure_data::adventure_action::AdventureAction;
use anyhow::Result;
use core_data::adventure_primitives::{AdventureOutcome, Coins, TilePosition};
use core_data::game_primitives::CardType;
use with_error::{fail, verify};

pub mod adventure_effect;
pub mod narrative_events;

/// Handles an incoming [AdventureAction] and produces a client response.
pub fn handle_adventure_action(state: &mut AdventureState, action: &AdventureAction) -> Result<()> {
    match action {
        AdventureAction::AbandonAdventure => handle_abandon_adventure(state),
        AdventureAction::VisitTileEntity(position) => handle_visit_tile(state, *position),
        AdventureAction::EndVisit => handle_end_visit(state),
        AdventureAction::DraftCard(index) => handle_draft_choice(state, *index),
        AdventureAction::BuyCard(index) => handle_buy_card(state, *index),
        AdventureAction::SetNarrativeStep(step) => {
            narrative_events::set_narrative_step(state, *step)
        }
        AdventureAction::ApplyNarrativeEffect(choice_index, effect_index) => {
            narrative_events::apply_narrative_effect(state, *choice_index, *effect_index)
        }
    }
}

fn handle_abandon_adventure(state: &mut AdventureState) -> Result<()> {
    state.outcome = Some(AdventureOutcome::Defeat);
    Ok(())
}

fn handle_visit_tile(state: &mut AdventureState, position: TilePosition) -> Result<()> {
    if let Some(effect) = &state.world_map.tile(position)?.on_visited {
        adventure_effect::apply(state, effect.clone(), None)?;
    }

    let tile = state.world_map.tile_mut(position)?;
    tile.on_visited = None;
    tile.icons.clear();
    Ok(())
}

fn handle_end_visit(state: &mut AdventureState) -> Result<()> {
    verify!(is_blocking_screen(state) != Some(true), "Cannot end visit on this screen");
    state.screens.pop();
    Ok(())
}

/// Returns Some(true) if the player cannot end a visit on the current screen
/// without taking some other game action.
fn is_blocking_screen(state: &mut AdventureState) -> Option<bool> {
    let screen = state.screens.current()?;
    match screen {
        AdventureScreen::Draft(_) => Some(true),
        _ => None,
    }
}

fn handle_draft_choice(state: &mut AdventureState, index: usize) -> Result<()> {
    let Some(AdventureScreen::Draft(data)) = state.screens.current() else {
        fail!("Expected active draft screen");
    };

    verify!(index < data.choices.len(), "Index out of bounds!");
    let choice = data.choices[index];

    let definition = rules::get(choice.card);
    if definition.card_type == CardType::Riftcaller {
        if !state.deck.schools.contains(&definition.school) {
            state.deck.schools.push(definition.school);
        }
        state.deck.identities.push(definition.variant());
    } else {
        state
            .deck
            .cards
            .entry(choice.card)
            .and_modify(|i| *i += choice.quantity)
            .or_insert(choice.quantity);
    }

    state.screens.pop();
    Ok(())
}

fn handle_buy_card(state: &mut AdventureState, index: usize) -> Result<()> {
    let Some(AdventureScreen::Shop(data)) = state.screens.current_mut() else {
        fail!("Expected active shop screen");
    };

    verify!(index < data.choices.len(), "Index out of bounds!");
    let choice = data.choices[index];
    verify!(!choice.sold, "Item already sold!");
    data.choices[index].sold = true;

    state
        .deck
        .cards
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
