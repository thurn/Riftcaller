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

use adventure_data::adventure::{CardChoice, ShopData};
use adventure_data::adventure_action::AdventureAction;
use player_data::PlayerState;

use core_ui::{icons, style};
use core_ui::action_builder::ActionBuilder;
use core_ui::animations::{
    self, AnimateStyle, AnimateToElement, CloneElement, DestroyElement, InterfaceAnimation,
};
use core_ui::button::Button;
use core_ui::full_screen_image::FullScreenImage;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use deck_card::{CardHeight, DeckCard};
use deck_card::deck_card_slot::DeckCardSlot;
use element_names::ElementName;
use panel_address::{Panel, PanelAddress};
use protos::spelldawn::{FlexAlign, FlexJustify, FlexVector2};
use protos::spelldawn::animate_element_style::Property;
use protos::spelldawn::game_command::Command;
use screen_overlay::ScreenOverlay;

pub struct ShopPanel<'a> {
    pub player: &'a PlayerState,
    pub address: PanelAddress,
    pub data: &'a ShopData,
}

impl<'a> Panel for ShopPanel<'a> {
    fn address(&self) -> PanelAddress {
        self.address
    }

    fn screen_overlay(&self) -> Option<Node> {
        ScreenOverlay::new(self.player)
            .show_close_button(Panels::close(self.address()).action(AdventureAction::EndVisit))
            .build()
    }
}

fn animate_card_to_deck(card_element: ElementName, pick_button: ElementName) -> impl Into<Command> {
    animations::combine(vec![
        animations::fade_out(pick_button),
        InterfaceAnimation::new()
            .start(card_element, CloneElement)
            .start(card_element, AnimateStyle::new(Property::Scale(FlexVector2 { x: 0.1, y: 0.1 })))
            .start(card_element, AnimateToElement::new(element_names::DECK_BUTTON))
            .insert(animations::default_duration(), card_element, DestroyElement),
    ])
}

fn shop_row(choices: &[CardChoice]) -> impl Component {
    Row::new("ShopRow")
        .style(
            Style::new()
                .flex_grow(1.0)
                .align_items(FlexAlign::FlexStart)
                .justify_content(FlexJustify::Center),
        )
        .children(choices.iter().enumerate().map(|(i, choice)| {
            let card_element = element_names::deck_card(choice.card);
            Column::new("ShopChoice")
                .style(Style::new().margin(Edge::All, 8.px()))
                .child(
                    DeckCardSlot::new(CardHeight::vh(40.0))
                        .layout(Layout::new().margin(Edge::All, 4.px()))
                        .card(
                            (!choice.sold).then(|| {
                                DeckCard::new(choice.card).quantity(Some(choice.quantity))
                            }),
                        ),
                )
                .child_node(if choice.sold {
                    Row::new("EmptyButton")
                        .style(
                            Style::new().height(88.px()).width(88.px()).margin(Edge::Top, 24.px()),
                        )
                        .build()
                } else {
                    let button = element_names::buy_card(choice.card);
                    Button::new(format!("{} {}", choice.cost, icons::COINS))
                        .name(button)
                        .layout(
                            Layout::new()
                                .margin(Edge::Horizontal, 8.px())
                                .margin(Edge::Top, 24.px()),
                        )
                        .action(
                            ActionBuilder::new()
                                .action(AdventureAction::BuyCard(i))
                                .update(animate_card_to_deck(card_element, button)),
                        )
                        .build()
                })
        }))
}

impl<'a> Component for ShopPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImage::new()
            .image(style::sprite(
                "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Store/SceneryStore_outside_1",
            ))
            .content(Column::new("ShopPanel").child(shop_row(&self.data.choices)))
            .build()
    }
}
