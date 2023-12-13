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

use adventure_data::adventure::{NarrativeEventChoice, NarrativeEventData, NarrativeEventStep};
use adventure_data::adventure_action::AdventureAction;
use core_data::adventure_primitives::NarrativeChoiceIndex;
use core_ui::actions::InterfaceAction;
use core_ui::design::{BackgroundColor, FontSize};
use core_ui::full_screen_image::FullScreenImage;
use core_ui::prelude::*;
use core_ui::style::{self, Corner};
use core_ui::text::Text;
use panel_address::{Panel, PanelAddress};
use player_data::PlayerState;
use protos::riftcaller::{
    FlexAlign, FlexJustify, FlexPosition, FlexVector3, TextAlign, WhiteSpace,
};

pub struct NarrativeEventPanel<'a> {
    pub player: &'a PlayerState,
    pub address: PanelAddress,
    pub data: &'a NarrativeEventData,
}

impl<'a> NarrativeEventPanel<'a> {
    fn introduction(&self) -> Column {
        self.container()
            .child(
                Text::new(self.data.description.clone())
                    .layout(
                        Layout::new()
                            .margin(Edge::Horizontal, 8.px())
                            .margin(Edge::Top, 4.px())
                            .margin(Edge::Bottom, 32.px()),
                    )
                    .font_size(FontSize::NarrativeText)
                    .text_align(TextAlign::MiddleLeft)
                    .white_space(WhiteSpace::Normal),
            )
            .child(Self::button_row(
                "Continue",
                AdventureAction::SetNarrativeStep(NarrativeEventStep::ViewChoices),
            ))
            .child(Self::button_row(
                "Flee",
                AdventureAction::SetNarrativeStep(NarrativeEventStep::ViewChoices),
            ))
    }

    fn view_choices(&self) -> Column {
        self.container().children(self.data.choices.iter().map(Self::narrative_choice)).child(
            Self::button_row(
                "Back",
                AdventureAction::SetNarrativeStep(NarrativeEventStep::Introduction),
            ),
        )
    }

    fn narrative_choice(choice: &NarrativeEventChoice) -> impl Component {
        Self::button_row(
            choice.choice_description.clone(),
            AdventureAction::SetNarrativeStep(NarrativeEventStep::SelectChoice(
                NarrativeChoiceIndex { value: 0 },
            )),
        )
    }

    fn button_row(
        text: impl Into<String>,
        action: impl InterfaceAction + 'static,
    ) -> impl Component {
        Row::new("ButtonRow")
            .style(
                Style::new()
                    .margin(Edge::Vertical, 4.px())
                    .padding(Edge::Horizontal, 8.px())
                    .background_color(BackgroundColor::NarrativeEventChoice)
                    .min_height(88.px())
                    .border_radius(Corner::All, 8.px()),
            )
            .hover_style(Style::new().background_color(BackgroundColor::NarrativeEventChoiceHover))
            .pressed_style(
                Style::new()
                    .background_color(BackgroundColor::NarrativeEventChoicePress)
                    .scale(FlexVector3 { x: 0.98, y: 0.98, z: 0.98 }),
            )
            .on_click(action)
            .child(
                Text::new(text)
                    .font_size(FontSize::NarrativeText)
                    .text_align(TextAlign::MiddleLeft)
                    .white_space(WhiteSpace::Normal),
            )
    }

    fn container(&self) -> Column {
        Column::new("NarrativeEventPanel").style(
            Style::new()
                .position_type(FlexPosition::Absolute)
                .position(Edge::Right, 16.px())
                .position(Edge::Bottom, 16.px())
                .width(400.px())
                .height(600.px())
                .background_color(BackgroundColor::NarrativeEventBackground)
                .border_radius(Corner::All, 8.px())
                .justify_content(FlexJustify::FlexStart)
                .align_items(FlexAlign::Stretch)
                .padding(Edge::All, 16.px()),
        )
    }
}

impl<'a> Panel for NarrativeEventPanel<'a> {
    fn address(&self) -> PanelAddress {
        self.address
    }
}

const BACKGROUND: &'static str = "Art/Tithi Luadthong/shutterstock_2018848967";

impl<'a> Component for NarrativeEventPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImage::new()
            .image(style::sprite_jpg(BACKGROUND))
            .disable_overlay(true)
            .content(if self.data.step == NarrativeEventStep::Introduction {
                self.introduction()
            } else {
                self.view_choices()
            })
            .build()
    }
}
