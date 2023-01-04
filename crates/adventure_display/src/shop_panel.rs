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

use anyhow::Result;
use core_ui::button::Button;
use core_ui::prelude::*;
use core_ui::update_element::ElementName;
use core_ui::{actions, icons, style, update_element};
use data::adventure::{CardChoice, ShopData, TileEntity, TilePosition};
use data::adventure_action::AdventureAction;
use data::player_data::PlayerData;
use deck_card::deck_card_slot::DeckCardSlot;
use deck_card::{CardHeight, DeckCard};
use panel_address::{Panel, PanelAddress};
use protos::spelldawn::{DestroyAnimationEffect, FlexAlign, FlexJustify};
use screen_overlay::ScreenOverlay;
use with_error::fail;

use crate::full_screen_image_panel::FullScreenImagePanel;

pub struct ShopPanel<'a> {
    position: TilePosition,
    player: &'a PlayerData,
    data: &'a ShopData,
}

impl<'a> ShopPanel<'a> {
    pub fn new(player: &'a PlayerData, position: TilePosition) -> Result<Self> {
        let TileEntity::Shop { data } = player.adventure()?.tile_entity(position)? else {
            fail!("Expected shop entity")
        };

        Ok(Self { position, player, data })
    }
}

impl<'a> Panel for ShopPanel<'a> {
    fn address(&self) -> PanelAddress {
        PanelAddress::Shop(self.position)
    }

    fn screen_overlay(&self) -> Option<Node> {
        ScreenOverlay::new(self.player).show_close_button(self.address()).build()
    }
}

fn shop_row(position: TilePosition, choices: &[CardChoice]) -> impl Component {
    Row::new("ShopRow")
        .style(
            Style::new()
                .flex_grow(1.0)
                .align_items(FlexAlign::FlexStart)
                .justify_content(FlexJustify::Center),
        )
        .children(choices.iter().enumerate().map(|(i, choice)| {
            let card_element = ElementName::new("Choice");
            Column::new("ShopChoice")
                .style(Style::new().margin(Edge::All, 8.px()))
                .child(
                    DeckCardSlot::new(CardHeight::vh(40.0))
                        .layout(Layout::new().margin(Edge::All, 4.px()))
                        .card((!choice.sold).then(|| {
                            DeckCard::new(choice.card)
                                .element_name(&card_element)
                                .quantity(choice.quantity)
                        })),
                )
                .child_node(if choice.sold {
                    Row::new("EmptyButton")
                        .style(
                            Style::new().height(88.px()).width(88.px()).margin(Edge::Top, 24.px()),
                        )
                        .build()
                } else {
                    let name = ElementName::new("Buy");
                    Button::new(format!("{} {}", choice.cost, icons::COINS))
                        .name(&name)
                        .layout(
                            Layout::new()
                                .margin(Edge::Horizontal, 8.px())
                                .margin(Edge::Top, 24.px()),
                        )
                        .action(actions::with_optimistic_update(
                            vec![
                                update_element::destroy(&name, DestroyAnimationEffect::FadeOut),
                                update_element::animate_to_position_and_destroy(
                                    &card_element,
                                    &element_names::DECK_BUTTON,
                                    DestroyAnimationEffect::Shrink,
                                ),
                            ],
                            AdventureAction::BuyCard(position, i),
                        ))
                        .build()
                })
        }))
}

impl<'a> Component for ShopPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImagePanel::new()
            .image(style::sprite(
                "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Store/SceneryStore_outside_1",
            ))
            .content(Column::new("ShopPanel").child(shop_row(self.position, &self.data.choices)))
            .build()
    }
}
