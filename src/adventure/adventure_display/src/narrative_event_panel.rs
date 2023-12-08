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

use adventure_data::adventure::NarrativeEventData;
use adventure_data::adventure_action::AdventureAction;
use core_ui::button::Button;
use core_ui::design::{BackgroundColor, FontSize};
use core_ui::full_screen_image::FullScreenImage;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::style::{self, Corner};
use core_ui::text::Text;
use panel_address::{Panel, PanelAddress};
use player_data::PlayerState;
use protos::riftcaller::{FlexAlign, FlexJustify, FlexPosition, TextAlign, WhiteSpace};
use screen_overlay::ScreenOverlay;

pub struct NarrativeEventPanel<'a> {
    pub player: &'a PlayerState,
    pub address: PanelAddress,
    pub data: &'a NarrativeEventData,
}

impl<'a> Panel for NarrativeEventPanel<'a> {
    fn address(&self) -> PanelAddress {
        self.address
    }

    fn screen_overlay(&self) -> Option<Node> {
        ScreenOverlay::new(self.player)
            .show_close_button(Panels::close(self.address()).action(AdventureAction::EndVisit))
            .build()
    }
}

const BACKGROUND: &'static str = "Art/Tithi Luadthong/shutterstock_2018848967";

impl<'a> Component for NarrativeEventPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImage::new()
            .image(style::sprite_jpg(BACKGROUND))
            .disable_overlay(true)
            .content(
                Column::new("NarrativeEventPanel")
                    .style(
                        Style::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Right, 16.px())
                            .position(Edge::Bottom, 16.px())
                            .width(400.px())
                            .background_color(BackgroundColor::NarrativeEventBackground)
                            .border_radius(Corner::All, 8.px())
                            .justify_content(FlexJustify::Center)
                            .align_items(FlexAlign::Center)
                            .padding(Edge::All, 16.px()),
                    )
                    .child(
                        Text::new(format!(
                            "As the dust and debris swirl around the jagged peaks, \
                    you find yourself face-to-face with the legendary Stormfeather Eagle, its eyes \
                    ablaze with a fierce intelligence.\n\nThe air crackles with the power of this mythical \
                    beast, and it's clear that only one of you will leave these heights as victor."
                        ))
                        .font_size(FontSize::Body)
                            .text_align(TextAlign::MiddleLeft)
                            .white_space(WhiteSpace::Normal),
                    )
                    .child(
                        Button::new("Continue").layout(Layout::new().margin(Edge::Top, 24.px())),
                    ),
            )
            .build()
    }
}
