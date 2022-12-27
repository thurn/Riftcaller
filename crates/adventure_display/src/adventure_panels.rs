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
use data::adventure::{TileEntity, TilePosition};
use data::player_data::PlayerData;
use panel_address::{Panel, PanelAddress};
use protos::spelldawn::{InterfacePanel, InterfacePanelAddress};
use with_error::{fail, WithError};

use crate::draft_prompt_panel::DraftPromptPanel;
use crate::explore_panel::ExplorePanel;
use crate::shop_prompt_panel::ShopPromptPanel;

/// Renders a panel for the entity at the provided [TilePosition].
pub fn render_tile_panel(
    position: TilePosition,
    player: &PlayerData,
    client_address: InterfacePanelAddress,
) -> Result<InterfacePanel> {
    let address = PanelAddress::TileEntity(position);
    let Some(adventure) = &player.adventure else {
        fail!("Expected active adventure");
    };

    let tile = adventure.tiles.get(&position).with_error(|| "Tile not found")?;
    let Some(entity) = &tile.entity else {
        // Entity does not exist, e.g. because it has been cleared after activation. This
        // is fine, just render nothing.
        return Ok(InterfacePanel { address: Some(client_address), node: None });
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
