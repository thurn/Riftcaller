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

//! Addresses for user interface panels

use adventure_data::adventure::TilePosition;
use core_data::game_primitives::{DeckId, Side};
use core_ui::panels::Panels;
use core_ui::prelude::Component;
use enum_kinds::EnumKind;
use game_data::card_name::CardMetadata;
use game_data::card_state::CardPosition;
use player_data::PlayerActivityKind;
use protos::spelldawn::{InterfacePanel, InterfacePanelAddress, Node};
use serde::{Deserialize, Serialize};
use serde_json::ser;

pub trait Panel: Component {
    fn address(&self) -> PanelAddress;

    fn close(&self) -> Panels {
        Panels::close(self.address())
    }

    /// Allows a custom screen overlay to be displayed while this panel is
    /// visible.
    fn screen_overlay(&self) -> Option<Node> {
        None
    }

    fn build_panel(self) -> Option<InterfacePanel>
    where
        Self: Sized,
    {
        let screen_overlay = self.screen_overlay();
        let address = self.address().into();
        self.build().map(|node| InterfacePanel {
            address: Some(address),
            node: Some(node),
            screen_overlay,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, EnumKind)]
#[enum_kind(StandardPanelKind)]
pub enum StandardPanel {
    MainMenu,
    About,
    Settings,
    SideSelect,
    Disclaimer,
    DebugPanel(PlayerActivityKind, Option<Side>),
    GameMenu,
    AdventureMenu,
    DeckEditorLoading,
    SetPlayerName(Side),
    DebugCreateCard(Side, CardMetadata),
    AddToZone { position: CardPosition, metadata: CardMetadata, turn_face_up: bool },
    ApplyScenario,
}

impl From<StandardPanel> for PanelAddress {
    fn from(value: StandardPanel) -> Self {
        PanelAddress::StandardPanel(value)
    }
}

impl From<StandardPanel> for InterfacePanelAddress {
    fn from(address: StandardPanel) -> Self {
        let a: PanelAddress = address.into();
        a.into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, EnumKind)]
#[enum_kind(PlayerPanelKind)]
pub enum PlayerPanel {
    DeckEditorPrompt,
    DeckEditor(DeckEditorData),
    BattleVictory,
    BattleDefeat,
    AdventureTile(TilePosition),
    AdventureOver,
}

impl From<PlayerPanel> for PanelAddress {
    fn from(value: PlayerPanel) -> Self {
        PanelAddress::PlayerPanel(value)
    }
}

impl From<PlayerPanel> for InterfacePanelAddress {
    fn from(address: PlayerPanel) -> Self {
        let a: PanelAddress = address.into();
        a.into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelAddress {
    StandardPanel(StandardPanel),
    PlayerPanel(PlayerPanel),
}

impl From<PanelAddress> for InterfacePanelAddress {
    fn from(address: PanelAddress) -> Self {
        Self {
            debug_string: match address {
                PanelAddress::StandardPanel(p) => {
                    let kind: StandardPanelKind = p.into();
                    format!("{kind:?}")
                }
                PanelAddress::PlayerPanel(p) => {
                    let kind: PlayerPanelKind = p.into();
                    format!("{kind:?}")
                }
            },
            serialized: ser::to_vec(&address).expect("Serialization failed"),
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CollectionBrowserFilters {
    pub offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeckEditorData {
    /// Identifes the deck being edited
    pub deck_id: DeckId,
    /// Current collection browser view
    pub collection_filters: CollectionBrowserFilters,
}

impl DeckEditorData {
    pub fn new(deck_id: DeckId) -> Self {
        Self { deck_id, collection_filters: CollectionBrowserFilters::default() }
    }
}
