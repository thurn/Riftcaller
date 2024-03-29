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

use adventure_data::adventure::BattleData;
use adventure_data::adventure_action::AdventureAction;
use core_data::game_primitives::{DeckId, School};
use game_data::player_name::PlayerId;
use player_data::PlayerState;
use user_action_data::{NewGameAction, NewGameDeck, UserAction};

use core_ui::button::Button;
use core_ui::design::{BackgroundColor, FontSize};
use core_ui::full_screen_image::FullScreenImage;
use core_ui::icons;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::style::{self, Corner};
use core_ui::text::Text;
use deck_card::CARD_ASPECT_RATIO;
use panel_address::{Panel, PanelAddress};
use protos::riftcaller::{FlexAlign, FlexJustify, FlexPosition};
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

const BACKGROUND: &'static str = "TPR/InfiniteEnvironments/meadow";

impl<'a> Component for BattlePanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImage::new()
            .image(style::sprite(BACKGROUND))
            .content(
                Column::new("BattlePanel")
                    .style(
                        Style::new()
                            .justify_content(FlexJustify::Center)
                            .align_items(FlexAlign::Center)
                            .margin(Edge::Bottom, 16.px()),
                    )
                    .child(
                        Text::new(format!("Battle vs. {}", self.data.opponent_name))
                            .font_size(FontSize::Headline),
                    )
                    .child(
                        Text::new(format!(
                            "Reward: {} <color=yellow>{}</color>",
                            self.data.reward,
                            icons::COINS
                        ))
                        .font_size(FontSize::Body),
                    )
                    .child(
                        Row::new("Schools")
                            .style(Style::new().margin(Edge::All, 16.px()))
                            .children(
                                self.data
                                    .opponent_deck
                                    .schools
                                    .iter()
                                    .map(|school| school_image(*school)),
                            ),
                    )
                    .child(
                        Button::new("Start Battle")
                            .layout(Layout::new().margin(Edge::All, 20.px()))
                            .min_width(400.px())
                            .action(UserAction::NewGame(NewGameAction {
                                deck: NewGameDeck::DeckId(DeckId::Adventure),
                                opponent: PlayerId::AI(self.data.opponent_id),
                                tutorial: false,
                                debug_options: None,
                            })),
                    ),
            )
            .build()
    }
}

fn school_image(school: School) -> Column {
    let width = 45.0;
    Column::new("School")
        .style(
            Style::new()
                .height(width.vh())
                .width((width * CARD_ASPECT_RATIO).vh())
                .margin(Edge::All, 16.px())
                .justify_content(FlexJustify::Center)
                .align_items(FlexAlign::Center)
                .background_image(assets::card_back(school)),
        )
        .child(
            Row::new("LabelBackground")
                .style(
                    Style::new()
                        .position_type(FlexPosition::Absolute)
                        .position(Edge::Bottom, 32.px())
                        .padding(Edge::Horizontal, 16.px())
                        .border_radius(Corner::All, 4.px())
                        .background_color(BackgroundColor::MediaOverlay),
                )
                .child(Text::new(school.displayed_name()).font_size(FontSize::SchoolLabel)),
        )
}
