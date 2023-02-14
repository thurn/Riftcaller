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

use adventure_data::adventure::{Coins, TilePosition};
use adventure_data::adventure_action::AdventureAction;
use core_ui::button::{Button, ButtonType};
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::prompt_panel::PromptPanel;
use core_ui::{icons, style};
use panel_address::{Panel, PanelAddress, PlayerPanel};
use user_action_data::UserAction;

pub struct ExplorePanel {
    pub cost: Coins,
    pub position: TilePosition,
}

impl Panel for ExplorePanel {
    fn address(&self) -> PanelAddress {
        PlayerPanel::TilePrompt(self.position).into()
    }
}

impl Component for ExplorePanel {
    fn build(self) -> Option<Node> {
        PromptPanel::new()
            .image(style::sprite("TPR/InfiniteEnvironments/meadow"))
            .prompt("To the north lie the flowering fields of the Kingdom of Edennes")
            .buttons(vec![
                Button::new(format!("Explore: {} {}", self.cost, icons::COINS))
                    .action(self.close().action(UserAction::AdventureAction(
                        AdventureAction::Explore(self.position),
                    )))
                    .layout(Layout::new().margin(Edge::All, 8.px())),
                Button::new("Close")
                    .button_type(ButtonType::Secondary)
                    .action(Panels::close(self.address()))
                    .layout(Layout::new().margin(Edge::All, 8.px())),
            ])
            .build()
    }
}
