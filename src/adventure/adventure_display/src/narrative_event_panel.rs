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
use adventure_data::adventure_action::{AdventureAction, NarrativeEffectIndex};
use adventure_data::adventure_effect_data::AdventureEffectData;
use core_data::adventure_primitives::NarrativeChoiceIndex;
use core_ui::actions::InterfaceAction;
use core_ui::design::{BackgroundColor, FontSize};
use core_ui::full_screen_image::FullScreenImage;
use core_ui::interface_animations;
use core_ui::prelude::*;
use core_ui::style::{self, Corner};
use core_ui::text::Text;
use game_data::card_name::CardVariant;
use panel_address::{Panel, PanelAddress};
use player_data::PlayerState;
use protos::riftcaller::game_command::Command;
use protos::riftcaller::{
    FlexAlign, FlexDisplayStyle, FlexJustify, FlexPosition, FlexVector3, InfoZoomCommand,
    TextAlign, WhiteSpace,
};

const CONTAINER_WIDTH: i32 = 400;
const CONTAINER_HEIGHT: i32 = 750;

pub struct NarrativeEventPanel<'a> {
    pub player: &'a PlayerState,
    pub address: PanelAddress,
    pub data: &'a NarrativeEventData,
}

impl<'a> NarrativeEventPanel<'a> {
    fn introduction(&self) -> Column {
        self.container()
            .child(Self::description(self.data.description.clone()))
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
        self.container().children(self.data.enumerate_choices().map(Self::narrative_choice)).child(
            Self::button_row(
                "Back",
                AdventureAction::SetNarrativeStep(NarrativeEventStep::Introduction),
            ),
        )
    }

    fn view_outcome(&self, choice_index: NarrativeChoiceIndex) -> Column {
        let choice = self.data.choice(choice_index);
        self.container()
            .child(Self::description(choice.result_description.clone()))
            .children(choice.enumerate_costs().map(|(effect_index, data)| {
                Self::apply_effect_row(data, choice_index, effect_index)
            }))
            .children(choice.enumerate_effects().map(|(effect_index, data)| {
                Self::apply_effect_row(data, choice_index, effect_index)
            }))
    }

    fn narrative_choice(
        (index, choice): (NarrativeChoiceIndex, &NarrativeEventChoice),
    ) -> impl Component {
        let known = Self::known_cards(choice);
        Self::button_row(
            choice.choice_description.clone(),
            AdventureAction::SetNarrativeStep(NarrativeEventStep::SelectChoice(index)),
        )
        .on_mouse_enter(vec![
            Some(interface_animations::set_displayed(
                element_names::narrative_outcome_tooltip(index),
                true,
            )),
            (!known.is_empty()).then(|| {
                // TODO: Handle multiple known cards
                Command::InfoZoom(InfoZoomCommand {
                    show: true,
                    card: Some(deck_card::card_view_for_variant(known[0])),
                })
            }),
        ])
        .on_mouse_leave(vec![
            interface_animations::set_displayed(
                element_names::narrative_outcome_tooltip(index),
                false,
            ),
            Command::InfoZoom(InfoZoomCommand { show: false, card: None }),
        ])
        .child(Self::outcome_tooltip(index, choice))
    }

    fn apply_effect_row(
        data: &AdventureEffectData,
        choice_index: NarrativeChoiceIndex,
        effect_index: NarrativeEffectIndex,
    ) -> impl Component {
        let is_cost = matches!(effect_index, NarrativeEffectIndex::Cost(..));
        if data.effect.is_immediate() {
            Row::new("ImmediateEffect").child(Self::effect_description(data, is_cost))
        } else {
            Self::button_row(
                Self::effect_text(data, is_cost),
                AdventureAction::ApplyNarrativeEffect(choice_index, effect_index),
            )
        }
    }

    /// Returns a list of all `known_card`s associated with costs & effects of
    /// this choice.
    fn known_cards(choice: &NarrativeEventChoice) -> Vec<CardVariant> {
        choice
            .costs
            .iter()
            .flat_map(|e| e.known_card)
            .chain(choice.rewards.iter().flat_map(|e| e.known_card))
            .collect()
    }

    fn outcome_tooltip(index: NarrativeChoiceIndex, choice: &NarrativeEventChoice) -> Column {
        Column::new(element_names::narrative_outcome_tooltip(index))
            .style(
                Style::new()
                    .display(FlexDisplayStyle::None)
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::Bottom, 0.px())
                    .position(Edge::Right, CONTAINER_WIDTH.px())
                    .background_color(BackgroundColor::NarrativeEventBackground)
                    .padding(Edge::All, 8.px())
                    .border_radius(Corner::All, 8.px()),
            )
            .children(choice.costs.iter().map(|choice| Self::effect_description(choice, true)))
            .children(choice.rewards.iter().map(|choice| Self::effect_description(choice, false)))
    }

    fn effect_text(data: &AdventureEffectData, cost: bool) -> String {
        let mut text =
            format!("<b>{}:</b> {}", if cost { "Cost" } else { "Reward" }, data.description);
        if let Some(name) = data.known_card {
            text = text.replace("{CardName}", &name.displayed_name());
        }
        text
    }

    fn effect_description(data: &AdventureEffectData, cost: bool) -> impl Component {
        Text::new(Self::effect_text(data, cost))
            .layout(Layout::new().margin(Edge::Vertical, 4.px()))
            .font_size(FontSize::NarrativeText)
            .text_align(TextAlign::MiddleLeft)
            .white_space(WhiteSpace::Normal)
    }

    fn description(text: impl Into<String>) -> impl Component {
        Text::new(text)
            .layout(
                Layout::new()
                    .margin(Edge::Horizontal, 8.px())
                    .margin(Edge::Top, 4.px())
                    .margin(Edge::Bottom, 32.px()),
            )
            .font_size(FontSize::NarrativeText)
            .text_align(TextAlign::MiddleLeft)
            .white_space(WhiteSpace::Normal)
    }

    fn button_row(text: impl Into<String>, action: impl InterfaceAction + 'static) -> Row {
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
                .width(CONTAINER_WIDTH.px())
                .min_height(CONTAINER_HEIGHT.px())
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
            .content(match self.data.step {
                NarrativeEventStep::Introduction => self.introduction(),
                NarrativeEventStep::ViewChoices => self.view_choices(),
                NarrativeEventStep::SelectChoice(index) => self.view_outcome(index),
            })
            .build()
    }
}
