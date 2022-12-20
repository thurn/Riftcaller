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

use data::adventure::TilePosition;
use data::player_name::PlayerId;
use data::primitives::{DeckIndex, GameId, School, Side};
use protos::spelldawn::{interface_panel_address, InterfacePanelAddress};
use serde::{Deserialize, Serialize};
use serde_json::ser;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelAddress {
    MainMenu,
    About,
    Settings,
    Disclaimer,
    AdventureMenu,
    SetPlayerName(Side),
    DeckEditor(DeckEditorData),
    CreateDeck(CreateDeckState),
    GameOver(GameOverData),
    TileEntity(TilePosition),

    /// Any screen with a mandatory choice which must be made within the context
    /// of an adventure.
    ///
    /// This is specifically for panels which *cannot* be directly opened by the
    /// user at any time but are adventure-state-dependent.
    AdventureChoice,
}

impl From<PanelAddress> for InterfacePanelAddress {
    fn from(address: PanelAddress) -> Self {
        Self {
            address_type: Some(interface_panel_address::AddressType::Serialized(
                ser::to_vec(&address).expect("Serialization failed"),
            )),
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CollectionBrowserFilters {
    pub offset: usize,
}

/// Identifies the current screen within the deck editor
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeckEditorData {
    /// Deck currently being viewed
    pub deck: Option<DeckIndex>,
    /// True if the detail options for the current deck (e.g. delete, rename)
    /// should be shown.
    pub show_edit_options: bool,
    /// Current collection browser view
    pub collection_filters: CollectionBrowserFilters,
}

/// Identifies which screen the user is on in the deck creation flow
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreateDeckState {
    PickSide,
    PickSchool(Side),
    PickName(Side, School),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameOverData {
    pub game_id: GameId,
    pub winner: PlayerId,
}
