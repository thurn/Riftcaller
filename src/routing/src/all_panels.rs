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

use adventure_data::adventure::AdventureState;
use game_data::primitives::{DeckId, Side};
use panel_address::{CollectionBrowserFilters, DeckEditorData, PlayerPanel, StandardPanel};
use player_data::{PlayerActivityKind, PlayerState};

/// Enumerates all standard panel addresses
pub fn standard_panels() -> Vec<StandardPanel> {
    vec![
        StandardPanel::MainMenu,
        StandardPanel::About,
        StandardPanel::Settings,
        StandardPanel::SideSelect,
        StandardPanel::Disclaimer,
        StandardPanel::DebugPanel(PlayerActivityKind::None),
        StandardPanel::DebugPanel(PlayerActivityKind::Adventure),
        StandardPanel::DebugPanel(PlayerActivityKind::PlayingGame),
        StandardPanel::GameMenu,
        StandardPanel::AdventureMenu,
        StandardPanel::SetPlayerName(Side::Champion),
        StandardPanel::SetPlayerName(Side::Overlord),
        StandardPanel::DeckEditorLoading,
    ]
}

/// Enumerates all player panel addresses
pub fn player_panels(player: &PlayerState) -> Vec<PlayerPanel> {
    let panels = vec![
        PlayerPanel::DeckEditorPrompt,
        PlayerPanel::AdventureOver,
        PlayerPanel::BattleVictory,
        PlayerPanel::BattleDefeat,
    ];
    if let Some(adventure) = &player.adventure {
        panels
            .into_iter()
            .chain(adventure.tiles.iter().filter_map(|(position, state)| {
                state.entity.as_ref().map(|_| PlayerPanel::AdventureTile(*position))
            }))
            .chain(add_deck_editor_panels(adventure))
            .collect()
    } else {
        panels
    }
}

fn add_deck_editor_panels(adventure: &AdventureState) -> impl Iterator<Item = PlayerPanel> {
    // Return deck editor pages for every 8 cards in the player's collection
    (0..=adventure.collection.len() / 8).map(|i| {
        PlayerPanel::DeckEditor(DeckEditorData {
            deck_id: DeckId::Adventure,
            collection_filters: CollectionBrowserFilters { offset: i * 8 },
        })
    })
}
