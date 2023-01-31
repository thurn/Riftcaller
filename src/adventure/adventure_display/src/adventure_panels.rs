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
use core_ui::full_screen_loading::FullScreenLoading;
use core_ui::prelude::*;
use game_data::adventure::{TileEntity, TilePosition};
use player_data::PlayerData;
use panel_address::{Panel, PanelAddress};
use protos::spelldawn::InterfacePanel;
use with_error::{fail, WithError};

use crate::draft_prompt_panel::DraftPromptPanel;
use crate::explore_panel::ExplorePanel;
use crate::shop_prompt_panel::ShopPromptPanel;

/// Renders an action prompt panel for the entity at the provided
/// [TilePosition].
pub fn render_tile_prompt_panel(
    position: TilePosition,
    player: &PlayerData,
) -> Result<Option<InterfacePanel>> {
    let address = PanelAddress::TilePrompt(position);
    let Some(adventure) = &player.adventure else {
        fail!("Expected active adventure");
    };

    let tile = adventure.tiles.get(&position).with_error(|| "Tile not found")?;
    let Some(entity) = &tile.entity else {
        // Entity does not exist, e.g. because it has been cleared after activation. This
        // is fine, just render nothing.
        return Ok(None)
    };

    Ok(match entity {
        TileEntity::Explore { cost, .. } => {
            ExplorePanel { cost: *cost, address, position }.build_panel()
        }
        TileEntity::Draft { cost, .. } => {
            DraftPromptPanel { cost: *cost, address, position }.build_panel()
        }
        TileEntity::Shop { .. } => ShopPromptPanel { address, position }.build_panel(),
    })
}

/// Renders the loading screen panel for the entity at the provided
/// [TilePosition], if any.
pub fn render_tile_loading_panel(
    position: TilePosition,
    player: &PlayerData,
) -> Result<Option<InterfacePanel>> {
    let node = match player.adventure()?.tile_entity(position)? {
        TileEntity::Explore { .. } => {
            FullScreenLoading::new("TPR/InfiniteEnvironments/meadow").build()
        }
        TileEntity::Draft { .. } => FullScreenLoading::new(
            "TPR/EnvironmentsHQ/Dungeons, Shrines & Altars/Images/MountainTomb/ScenerySnowMountain_1",
        )
        .build(),
        TileEntity::Shop { .. } => FullScreenLoading::new(
            "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Store/SceneryStore_outside_1",
        )
        .build(),
    };

    Ok(Some(InterfacePanel {
        address: Some(PanelAddress::TileLoading(position).into()),
        node,
        screen_overlay: None,
    }))
}
