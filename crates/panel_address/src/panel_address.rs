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

use core_ui::prelude::Component;
use data::adventure::TilePosition;
use data::player_name::PlayerId;
use data::primitives::{DeckId, GameId, Side};
use protos::spelldawn::{InterfacePanel, InterfacePanelAddress, Node};
use serde::{Deserialize, Serialize};
use serde_json::ser;

pub trait Panel: Component {
    fn address(&self) -> PanelAddress;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelAddress {
    MainMenu,
    About,
    Settings,
    Disclaimer,
    DebugPanel,
    GameMenu,
    AdventureMenu,
    SetPlayerName(Side),
    DeckEditorPrompt,
    DeckEditorLoading,
    DeckEditor(DeckEditorData),
    GameOver(GameOverData),
    TileLoading(TilePosition),
    TilePrompt(TilePosition),
    DraftCard,
    Shop(TilePosition),
    AdventureOver,
}

impl From<PanelAddress> for InterfacePanelAddress {
    fn from(address: PanelAddress) -> Self {
        Self { serialized: ser::to_vec(&address).expect("Serialization failed") }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameOverData {
    pub game_id: GameId,
    pub winner: PlayerId,
}
