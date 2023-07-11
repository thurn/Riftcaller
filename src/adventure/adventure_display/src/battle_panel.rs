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

use adventure_data::adventure::BattleData;
use adventure_data::adventure_action::AdventureAction;
use core_ui::full_screen_image::FullScreenImage;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::style;
use panel_address::{Panel, PanelAddress};
use player_data::PlayerState;
use screen_overlay::ScreenOverlay;

pub struct BattlePanel<'a> {
    pub player: &'a PlayerState,
    pub address: PanelAddress,
    pub data: &'a BattleData,
}

impl<'a> Panel for BattlePanel<'a> {
    fn address(&self) -> PanelAddress {
        self.address
    }

    fn screen_overlay(&self) -> Option<Node> {
        ScreenOverlay::new(self.player)
            .show_close_button(Panels::close(self.address()).action(AdventureAction::EndVisit))
            .build()
    }
}

impl<'a> Component for BattlePanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImage::new()
            .image(style::sprite(
                "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Store/SceneryStore_outside_1",
            ))
            .build()
    }
}
