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
use core_ui::{actions, icons, panel, style, update_element};
use data::adventure::{Coins, TilePosition};
use data::adventure_actions::AdventureAction;
use data::user_actions::UserAction;
use panel_address::PanelAddress;

use crate::tile_prompt_panel::TilePromptPanel;

pub struct DraftPromptPanel {
    pub cost: Coins,
    pub address: PanelAddress,
    pub position: TilePosition,
}

impl Component for DraftPromptPanel {
    fn build(self) -> Option<Node> {
        TilePromptPanel::new()
            .image(style::sprite("TPR/EnvironmentsHQ/mountain"))
            .prompt("An expedition into these mountain ruins could provide a valuable treasure")
            .buttons(vec![
                Button::new(format!("Draft: {} {}", self.cost, icons::COINS))
                    .action(actions::with_optimistic_update(
                        vec![update_element::clear(TilePromptPanel::content_name())],
                        UserAction::AdventureAction(AdventureAction::TileAction(self.position)),
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
