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

use core_ui::button::{Button, ButtonType};
use core_ui::prelude::*;
use core_ui::{panel, style};
use data::adventure::TilePosition;
use panel_address::PanelAddress;

use crate::adventure_loading::AdventureLoading;
use crate::tile_prompt_panel::TilePromptPanel;

pub struct ShopPromptPanel {
    pub address: PanelAddress,
    pub position: TilePosition,
}

impl Component for ShopPromptPanel {
    fn build(self) -> Option<Node> {
        TilePromptPanel::new()
            .image(style::sprite("TPR/EnvironmentsHQ/EnvironmentsHQ2/shop"))
            .prompt("Walking through town, you come upon the illuminated windows of a shop stocked with magical wares")
            .buttons(vec![
                Button::new("Continue")
                    .action(panel::transition(
                            self.address,
                            PanelAddress::Shop(self.position),
                            AdventureLoading::new("TPR/EnvironmentsHQ/EnvironmentsHQ2/shop"),
                        ))
                    .layout(Layout::new().margin(Edge::All, 8.px())),
                Button::new("Close")
                    .button_type(ButtonType::Secondary)
                    .action(panel::close(self.address))
                    .layout(Layout::new().margin(Edge::All, 8.px())),
            ])
            .build()
    }
}
