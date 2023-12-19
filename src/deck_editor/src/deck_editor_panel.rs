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

use adventure_data::adventure_effect_data::DeckCardEffect;
use core_ui::full_screen_image::FullScreenImage;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use core_ui::style;
use deck_card::deck_card_slot::DeckCardSlot;
use deck_card::{CardHeight, DeckCard};
use game_data::card_name::CardVariant;
use panel_address::{Panel, PanelAddress, PlayerPanel};
use player_data::PlayerState;
use protos::riftcaller::{
    FlexAlign, FlexDirection, FlexJustify, ScrollBarVisibility, TouchScrollBehavior,
};
use screen_overlay::ScreenOverlay;

pub struct DeckEditorPanel<'a> {
    /// Player state
    pub player: &'a PlayerState,
    /// Optionally an effect that can be applied to the cards being viewed.
    ///
    /// If specified, a button is displayed below each card allowing the player
    /// to apply this effect.
    pub action: Option<DeckCardEffect>,
}

impl<'a> DeckEditorPanel<'a> {
    fn card_row(cards: &[(&CardVariant, &u32)]) -> impl Component {
        let empty_slots = 5usize.saturating_sub(cards.len());
        Row::new("CardRow")
            .style(
                Style::new()
                    .flex_grow(1.0)
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center),
            )
            .children(cards.iter().map(|(n, quantity)| Self::collection_card(**n, **quantity)))
            .children((0..empty_slots).map(|_| {
                DeckCardSlot::new(CardHeight::vh(36.0))
                    .layout(Layout::new().margin(Edge::All, 16.px()))
            }))
    }

    fn collection_card(variant: CardVariant, quantity: u32) -> impl Component {
        DeckCardSlot::new(CardHeight::vh(36.0))
            .layout(Layout::new().margin(Edge::All, 16.px()))
            .card(Some(DeckCard::new(variant).quantity(Some(quantity))))
    }
}
impl<'a> Panel for DeckEditorPanel<'a> {
    fn address(&self) -> PanelAddress {
        PlayerPanel::DeckEditor(self.action).into()
    }

    fn screen_overlay(&self) -> Option<Node> {
        ScreenOverlay::new(self.player)
            .show_deck_button(false)
            .show_close_button(Panels::close(self.address()))
            .build()
    }
}
impl<'a> Component for DeckEditorPanel<'a> {
    fn build(self) -> Option<Node> {
        let deck = self.player.adventure.as_ref()?.deck.cards.iter().collect::<Vec<_>>();
        let chunks = deck.chunks(5);
        FullScreenImage::new()
            .image(style::sprite(
                "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Library/SceneryLibrary_inside_1",
            ))
            .content(
                ScrollView::new("DeckEditor")
                    .mouse_wheel_scroll_size(1000.0)
                    .horizontal_scrollbar_visibility(ScrollBarVisibility::Hidden)
                    .vertical_scrollbar_visibility(ScrollBarVisibility::Hidden)
                    .touch_scroll_behavior(TouchScrollBehavior::Clamped)
                    .scroll_deceleration_rate(0.0)
                    .style(
                        Style::new()
                            .flex_direction(FlexDirection::Column)
                            .flex_grow(1.0)
                            .align_items(FlexAlign::Center)
                            .justify_content(FlexJustify::Center),
                    )
                    .children(chunks.map(Self::card_row)),
            )
            .build()
    }
}
