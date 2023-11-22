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

use adventure_data::adventure::{DraftContext, DraftData};
use adventure_data::adventure_action::AdventureAction;
use core_data::game_primitives::Milliseconds;

use core_ui::action_builder::ActionBuilder;
use core_ui::button::Button;
use core_ui::design::FontSize;
use core_ui::full_screen_image::FullScreenImage;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::style;
use core_ui::text::Text;
use deck_card::{CardHeight, DeckCard};
use panel_address::{Panel, PanelAddress};
use protos::spelldawn::FlexJustify;

const BACKGROUND: &'static str =
    "TPR/EnvironmentsHQ/Dungeons, Shrines & Altars/Images/MountainTomb/ScenerySnowMountain_1";

pub struct DraftPanel<'a> {
    pub address: PanelAddress,
    pub data: &'a DraftData,
}

impl<'a> Panel for DraftPanel<'a> {
    fn address(&self) -> PanelAddress {
        self.address
    }
}

fn draft_title(data: &DraftData) -> Option<String> {
    Some(
        match data.context.as_ref()? {
            DraftContext::StartingRiftcaller => "Pick a starting Riftcaller:",
        }
        .to_string(),
    )
}

fn custom_button_label(data: &DraftData) -> Option<String> {
    Some(
        match data.context.as_ref()? {
            DraftContext::StartingRiftcaller => "Start",
        }
        .to_string(),
    )
}

impl<'a> Component for DraftPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImage::new()
            .image(style::sprite(BACKGROUND))
            .content(
                Column::new("DraftPanel")
                    .child(
                        draft_title(self.data)
                            .map(|title| Text::new(title).font_size(FontSize::Headline)),
                    )
                    .child(
                        Row::new("DraftChoices")
                            .style(Style::new().justify_content(FlexJustify::Center))
                            .children(self.data.choices.iter().enumerate().map(|(i, choice)| {
                                let button = element_names::draft_card(choice.card);
                                Column::new("Choice")
                                    .style(Style::new().margin(Edge::All, 32.px()))
                                    .child(
                                        DeckCard::new(choice.card)
                                            .layout(Layout::new().margin(Edge::All, 8.px()))
                                            .reveal_delay(Some(Milliseconds(
                                                300 + (i as u32 * 300),
                                            )))
                                            .height(CardHeight::vh(50.0)),
                                    )
                                    .child(
                                        Button::new(
                                            custom_button_label(self.data)
                                                .unwrap_or_else(|| "Pick".to_string()),
                                        )
                                        .name(button)
                                        .layout(
                                            Layout::new()
                                                .margin(Edge::Horizontal, 8.px())
                                                .margin(Edge::Top, 16.px()),
                                        )
                                        .action(
                                            ActionBuilder::new()
                                                .action(AdventureAction::DraftCard(i))
                                                .update(Panels::close(self.address())),
                                        ),
                                    )
                            })),
                    ),
            )
            .build()
    }
}
