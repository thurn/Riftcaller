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

pub mod card_generator;

use anyhow::Result;
use data::adventure::{AdventureChoiceScreen, AdventureState, TileEntity, TilePosition};
use data::player_data::PlayerData;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{LoadSceneCommand, SceneLoadMode};
use with_error::{fail, verify, WithError};

pub fn handle_abandon_adventure(state: &mut AdventureState) -> Result<()> {
    state.choice_screen = Some(AdventureChoiceScreen::AdventureOver);
    Ok(())
}

pub fn handle_leave_adventure(state: &mut PlayerData) -> Result<Vec<Command>> {
    state.adventure = None;
    Ok(vec![Command::LoadScene(LoadSceneCommand {
        scene_name: "Main".to_string(),
        mode: SceneLoadMode::Single.into(),
        skip_if_current: true,
    })])
}

pub fn handle_tile_action(state: &mut AdventureState, position: TilePosition) -> Result<()> {
    verify_no_mandatory_choice(state)?;
    let tile = state.tiles.get_mut(&position).with_error(|| "Tile not found")?;

    match tile.entity.as_ref().with_error(|| "No action for tile")? {
        TileEntity::Explore { region, cost } => {
            state.coins -= *cost;
            state.revealed_regions.insert(*region);
            tile.entity = None;
        }
        TileEntity::Draft { cost } => {
            state.coins -= *cost;
            tile.entity = None;
            let draft_data = card_generator::draft_choices(state);
            state.choice_screen = Some(AdventureChoiceScreen::Draft(draft_data));
        }
        TileEntity::Shop => {}
    }

    Ok(())
}

pub fn handle_draft(state: &mut AdventureState, index: usize) -> Result<()> {
    let Some(AdventureChoiceScreen::Draft(data)) = &state.choice_screen else {
        fail!("No active draft!")
    };

    let choice = data.choices.get(index).with_error(|| "Choice index out of bounds")?;
    state
        .collection
        .entry(choice.card)
        .and_modify(|i| *i += choice.quantity)
        .or_insert(choice.quantity);
    state.choice_screen = None;
    Ok(())
}

/// Other adventure actions cannot be take while a choice screen like 'draft a
/// card' is being displayed.
fn verify_no_mandatory_choice(state: &AdventureState) -> Result<()> {
    verify!(state.choice_screen.is_none(), "Mandatory choice screen is active!");
    Ok(())
}
