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

use core_ui::prelude::*;
use core_ui::{actions, panel};
use data::adventure_action::AdventureAction;
use data::user_actions::UserAction;
use panel_address::PanelAddress;

use crate::button_menu::ButtonMenu;

#[derive(Default)]
pub struct AdventureMenu {}

impl AdventureMenu {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for AdventureMenu {
    fn build(self) -> Option<Node> {
        ButtonMenu::new(PanelAddress::AdventureMenu)
            .button(
                "Abandon Adventure",
                actions::close_and(
                    PanelAddress::AdventureMenu,
                    UserAction::AdventureAction(AdventureAction::AbandonAdventure),
                ),
            )
            .button("Settings", panel::open(PanelAddress::Settings))
            .build()
    }
}
