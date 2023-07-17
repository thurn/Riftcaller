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

use core_ui::button::{Button, ButtonType};
use core_ui::design::FontSize;
use core_ui::panel_window::PanelWindow;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use core_ui::text::Text;
use panel_address::{Panel, PanelAddress, PlayerPanel};
use protos::spelldawn::{FlexAlign, FlexJustify, WhiteSpace};
use user_action_data::{GameOutcome, UserAction};

#[derive(Debug)]
pub struct BattleDefeatPanel {}

impl Panel for BattleDefeatPanel {
    fn address(&self) -> PanelAddress {
        PlayerPanel::BattleDefeat.into()
    }
}

impl Component for BattleDefeatPanel {
    fn build(self) -> Option<Node> {
        PanelWindow::new(self.address(), 512.px(), 350.px())
            .title("Game Over")
            .content(
                Column::new("Buttons")
                    .style(
                        Style::new()
                            .width(100.pct())
                            .align_items(FlexAlign::Stretch)
                            .justify_content(FlexJustify::Center),
                    )
                    .child(
                        Text::new("Your adventure ends here, wanderer.")
                            .white_space(WhiteSpace::Normal)
                            .font_size(FontSize::Headline),
                    )
                    .child(
                        Button::new("Main Menu")
                            .action(
                                Panels::close(self.address())
                                    .action(UserAction::LeaveGame(GameOutcome::Defeat)),
                            )
                            .button_type(ButtonType::Primary)
                            .width_mode(WidthMode::Flexible)
                            .layout(Layout::new().margin(Edge::All, 16.px())),
                    ),
            )
            .build()
    }
}
