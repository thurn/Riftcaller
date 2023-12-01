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

//! Panel shown at the end of a game summarizing the result

use adventure_data::adventure::BattleData;
use core_ui::button::{Button, ButtonType};
use core_ui::design::FontSize;
use core_ui::icons;
use core_ui::panel_window::PanelWindow;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::style::WidthMode;
use core_ui::text::Text;
use panel_address::{Panel, PanelAddress, PlayerPanel};
use player_data::PlayerState;
use protos::riftcaller::{FlexAlign, FlexJustify};
use user_action_data::{GameOutcome, UserAction};

#[derive(Debug)]
pub struct BattleVictoryPanel<'a> {
    data: Option<&'a BattleData>,
}

impl<'a> BattleVictoryPanel<'a> {
    pub fn new(player: &'a PlayerState) -> Self {
        Self { data: player_data::current_battle(player) }
    }
}

impl<'a> Panel for BattleVictoryPanel<'a> {
    fn address(&self) -> PanelAddress {
        PlayerPanel::BattleVictory.into()
    }
}

impl<'a> Component for BattleVictoryPanel<'a> {
    fn build(self) -> Option<Node> {
        let content = if let Some(data) = self.data {
            Column::new("Buttons")
                .style(
                    Style::new()
                        .width(100.pct())
                        .align_items(FlexAlign::Stretch)
                        .justify_content(FlexJustify::Center),
                )
                .child(
                    Text::new(format!(
                        "Gained {} <color=yellow>{}</color>",
                        data.reward,
                        icons::COINS
                    ))
                    .font_size(FontSize::Headline),
                )
                .child(
                    Button::new("Continue")
                        .action(
                            Panels::close(self.address())
                                .action(UserAction::LeaveGame(GameOutcome::Victory)),
                        )
                        .button_type(ButtonType::Primary)
                        .width_mode(WidthMode::Flexible)
                        .layout(Layout::new().margin(Edge::All, 16.px())),
                )
        } else {
            Column::new("Buttons")
                .style(
                    Style::new()
                        .width(100.pct())
                        .align_items(FlexAlign::Stretch)
                        .justify_content(FlexJustify::Center),
                )
                .child(
                    Button::new("Main Menu")
                        .action(
                            Panels::close(self.address())
                                .action(UserAction::LeaveGame(GameOutcome::Victory)),
                        )
                        .button_type(ButtonType::Primary)
                        .width_mode(WidthMode::Flexible)
                        .layout(Layout::new().margin(Edge::All, 16.px())),
                )
        };

        PanelWindow::new(self.address(), 512.px(), 350.px())
            .title(if self.data.is_some() { "Rewards" } else { "You Win!" })
            .content(content)
            .build()
    }
}
