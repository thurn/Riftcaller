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

use adventure_data::adventure::TileEntity;
use anyhow::Result;
use core_data::adventure_primitives::TilePosition;
use player_data::PlayerState;

use panel_address::{Panel, PanelAddress, PlayerPanel};
use protos::spelldawn::InterfacePanel;
use with_error::WithError;

use crate::battle_panel::BattlePanel;
use crate::draft_panel::DraftPanel;
use crate::shop_panel::ShopPanel;

/// Builds an [InterfacePanel] for the adventure world map entity at the
/// specified position. Returns an error if no such entity exists.
pub fn tile_entity_panel(
    player: &PlayerState,
    position: TilePosition,
) -> Result<Option<InterfacePanel>> {
    let state = player.adventure()?;
    let address = PanelAddress::PlayerPanel(PlayerPanel::AdventureTile(position));
    Ok(match state.tile(position)?.entity.as_ref().with_error(|| "Expected tile entity")? {
        TileEntity::Draft(data) => DraftPanel { address, data }.build_panel(),
        TileEntity::Shop(data) => ShopPanel { player, address, data }.build_panel(),
        TileEntity::Battle(data) => BattlePanel { player, address, data }.build_panel(),
    })
}
