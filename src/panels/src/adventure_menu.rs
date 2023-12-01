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

use adventure_data::adventure_action::AdventureAction;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use panel_address::{Panel, PanelAddress, StandardPanel};
use user_action_data::UserAction;

use crate::button_menu::ButtonMenu;

#[derive(Default)]
pub struct AdventureMenu {}

impl AdventureMenu {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Panel for AdventureMenu {
    fn address(&self) -> PanelAddress {
        StandardPanel::AdventureMenu.into()
    }
}

impl Component for AdventureMenu {
    fn build(self) -> Option<Node> {
        ButtonMenu::new(self.address())
            .button(
                "Abandon Adventure",
                Panels::close(self.address())
                    .action(UserAction::AdventureAction(AdventureAction::AbandonAdventure)),
            )
            .button("Settings", Panels::open(StandardPanel::Settings))
            .build()
    }
}
