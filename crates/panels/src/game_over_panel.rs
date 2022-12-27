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

//! Panel shown at the end of a game summarizing the result

use core_ui::actions;
use core_ui::button::{Button, ButtonType};
use core_ui::panel::Panel;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use data::player_data::PlayerData;
use data::user_actions::UserAction;
use panel_address::{PanelAddress, PanelType};
use protos::spelldawn::{FlexAlign, FlexJustify};

#[derive(Debug)]
pub struct GameOverPanel<'a> {
    pub address: PanelAddress,
    pub player: &'a PlayerData,
}

impl<'a> PanelType for GameOverPanel<'a> {}

impl<'a> Component for GameOverPanel<'a> {
    fn build(self) -> Option<Node> {
        Panel::new(self.address, 512.px(), 350.px())
            .content(
                Column::new("Buttons")
                    .style(
                        Style::new()
                            .width(100.pct())
                            .align_items(FlexAlign::Stretch)
                            .justify_content(FlexJustify::Center),
                    )
                    .child(
                        Button::new("Main Menu")
                            .action(actions::close_and(self.address, UserAction::LeaveGame))
                            .button_type(ButtonType::Primary)
                            .width_mode(WidthMode::Flexible)
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    ),
            )
            .build()
    }
}
