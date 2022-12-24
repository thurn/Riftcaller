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
use core_ui::{actions, icons, style};
use data::adventure::{CardChoice, ShopData, TileEntity, TilePosition};
use data::adventure_action::AdventureAction;
use data::player_data::PlayerData;
use deck_card::{CardHeight, DeckCard};
use panel_address::PanelAddress;
use protos::spelldawn::{FlexAlign, FlexJustify};
use with_error::fail;

use crate::full_screen_image_panel::FullScreenImagePanel;

pub struct ShopPanel<'a> {
    data: &'a ShopData,
}

impl<'a> ShopPanel<'a> {
    pub fn new_from_player(player: &'a PlayerData, position: TilePosition) -> Result<Self> {
        let TileEntity::Shop { data } = player.adventure()?.tile_entity(position)? else {
            fail!("Expected shop entity")
        };

        Ok(Self { data })
    }
}

fn shop_row<'a>(choices: impl Iterator<Item = &'a CardChoice>) -> impl Component {
    Row::new("ShopRow")
        .style(
            Style::new()
                .flex_grow(1.0)
                .align_items(FlexAlign::Center)
                .justify_content(FlexJustify::Center),
        )
        .children(choices.enumerate().map(|(i, choice)| {
            Column::new("ShopChoice")
                .style(Style::new().margin(Edge::All, 8.px()))
                .child(
                    DeckCard::new(choice.card)
                        .quantity(choice.quantity)
                        .layout(Layout::new().margin(Edge::All, 8.px()))
                        .height(CardHeight::vh(40.0)),
                )
                .child(
                    Button::new(format!("{} {}", choice.cost, icons::COINS))
                        .layout(
                            Layout::new()
                                .margin(Edge::Horizontal, 8.px())
                                .margin(Edge::Top, 24.px()),
                        )
                        .action(actions::close_and(
                            PanelAddress::DraftCard,
                            AdventureAction::DraftCard(i),
                        )),
                )
        }))
}

impl<'a> Component for ShopPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImagePanel::new()
            .image(style::sprite("TPR/EnvironmentsHQ/EnvironmentsHQ2/shop"))
            .content(Column::new("ShopPanel").child(shop_row(self.data.choices.iter())))
            .build()
    }
}
