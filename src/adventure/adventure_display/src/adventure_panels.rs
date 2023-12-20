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

use adventure_data::adventure::AdventureScreen;
use anyhow::Result;
use deck_editor::deck_editor_panel::DeckEditorPanel;
use panel_address::{Panel, PanelAddress, PlayerPanel};
use player_data::PlayerState;
use protos::riftcaller::InterfacePanel;

use crate::battle_panel::BattlePanel;
use crate::draft_panel::DraftPanel;
use crate::narrative_event_panel::NarrativeEventPanel;
use crate::shop_panel::ShopPanel;

/// Builds an [InterfacePanel] for the current adventure screen.
pub fn tile_entity_panel(player: &PlayerState, index: usize) -> Result<Option<InterfacePanel>> {
    let state = player.adventure()?;
    let Some(screen) = state.screens.get(index) else {
        return Ok(None);
    };
    build_panel(player, screen, index)
}

fn build_panel(
    player: &PlayerState,
    screen: &AdventureScreen,
    count: usize,
) -> Result<Option<InterfacePanel>> {
    let address = PanelAddress::PlayerPanel(PlayerPanel::AdventureScreen(count));
    Ok(match screen {
        AdventureScreen::Draft(data) => DraftPanel { address, data }.build_panel(),
        AdventureScreen::Shop(data) => ShopPanel { player, address, data }.build_panel(),
        AdventureScreen::Battle(data) => BattlePanel { player, address, data }.build_panel(),
        AdventureScreen::NarrativeEvent(data) => {
            NarrativeEventPanel { player, address, data }.build_panel()
        }
        AdventureScreen::ApplyDeckEffect(selector, effect) => {
            DeckEditorPanel { address, player, effect: Some(*effect), filter: Some(*selector) }
                .build_panel()
        }
    })
}
