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
use core_ui::full_screen_loading::FullScreenLoading;
use core_ui::prelude::*;
use core_ui::prompt_panel::PromptPanel;
use core_ui::{actions, icons, panels, style};
use data::adventure::{Coins, TilePosition};
use data::adventure_action::AdventureAction;
use data::user_actions::UserAction;
use panel_address::{Panel, PanelAddress};

pub struct DraftPromptPanel {
    pub cost: Coins,
    pub address: PanelAddress,
    pub position: TilePosition,
}

impl Panel for DraftPromptPanel {
    fn address(&self) -> PanelAddress {
        PanelAddress::TilePrompt(self.position)
    }
}

impl Component for DraftPromptPanel {
    fn build(self) -> Option<Node> {
        PromptPanel::new()
            .image(style::sprite("TPR/EnvironmentsHQ/Dungeons, Shrines & Altars/Images/MountainTomb/ScenerySnowMountain_1"))
            .prompt("An expedition into these mountain ruins could provide a valuable treasure")
            .buttons(vec![
                Button::new(format!("Draft: {} {}", self.cost, icons::COINS))
                    .action(actions::with_optimistic_update(
                        panels::close_and_wait_for(
                            self.address,
                            PanelAddress::DraftCard,
                            FullScreenLoading::new("TPR/EnvironmentsHQ/Dungeons, Shrines & Altars/Images/MountainTomb/ScenerySnowMountain_1"),
                        ),
                        UserAction::AdventureAction(AdventureAction::InitiateDraft(self.position)),
                    ))
                    .layout(Layout::new().margin(Edge::All, 8.px())),
                Button::new("Close")
                    .button_type(ButtonType::Secondary)
                    .action(panels::close(self.address))
                    .layout(Layout::new().margin(Edge::All, 8.px())),
            ])
            .build()
    }
}
