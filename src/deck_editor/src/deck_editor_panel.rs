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

use adventure_data::adventure::CardFilter;
use adventure_data::adventure_effect_data::{DeckCardAction, DeckCardEffect};
use adventure_generator::card_filter;
use core_ui::button::{Button, ButtonType};
use core_ui::design::FontSize;
use core_ui::full_screen_image::FullScreenImage;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use core_ui::style;
use core_ui::text::Text;
use deck_card::deck_card_slot::DeckCardSlot;
use deck_card::{CardHeight, DeckCard};
use game_data::card_name::CardVariant;
use panel_address::{Panel, PanelAddress};
use player_data::PlayerState;
use protos::riftcaller::{
    FlexAlign, FlexDirection, FlexJustify, FlexPosition, ScrollBarVisibility, TouchScrollBehavior,
};
use screen_overlay::ScreenOverlay;

pub struct DeckEditorPanel<'a> {
    /// Address of this panel
    pub address: PanelAddress,
    /// Player state
    pub player: &'a PlayerState,
    /// Optionally an effect that can be applied to the cards being viewed.
    ///
    /// If specified, a button is displayed below each card allowing the player
    /// to apply this effect.
    pub effect: Option<DeckCardEffect>,
    /// Optionally, a selector for which set of cards can have [Self::effect]
    /// applied to them.
    ///
    /// If not specified, all cards can be picked.
    pub filter: Option<CardFilter>,
}

impl<'a> DeckEditorPanel<'a> {
    fn card_section(&self, mut cards: Vec<(&CardVariant, Option<&u32>)>) -> Column {
        cards.sort_by_key(|(variant, _)| {
            let definition = rules::get(**variant);
            (definition.card_type, definition.cost.mana, definition.name.displayed_name())
        });
        let chunks = cards.chunks(5);
        Column::new("EditorSection").children(chunks.map(|c| self.card_row(c)))
    }

    fn card_row(&self, cards: &[(&CardVariant, Option<&u32>)]) -> impl Component {
        let empty_slots = 5usize.saturating_sub(cards.len());
        Row::new("CardRow")
            .style(
                Style::new()
                    .flex_grow(1.0)
                    .align_items(FlexAlign::FlexStart)
                    .justify_content(FlexJustify::Center)
                    .margin(Edge::Bottom, 32.px()),
            )
            .children(
                cards.iter().map(|(n, quantity)| self.collection_card(**n, quantity.copied())),
            )
            .children((0..empty_slots).map(|_| {
                DeckCardSlot::new(CardHeight::vh(36.0))
                    .layout(Layout::new().margin(Edge::All, 16.px()))
            }))
    }

    fn collection_card(&self, variant: CardVariant, quantity: Option<u32>) -> impl Component {
        Column::new("CollectionCard")
            .style(Style::new().margin(Edge::All, 16.px()))
            .child(
                DeckCardSlot::new(CardHeight::vh(36.0))
                    .card(Some(DeckCard::new(variant).quantity(quantity))),
            )
            .child(self.action_button(variant, quantity))
    }

    fn action_button(&self, variant: CardVariant, quantity: Option<u32>) -> impl Component {
        self.deck_card_effect(variant, quantity).map(|effect| {
            Button::new(match effect.action {
                DeckCardAction::DuplicateTo3Copies => "Duplicate",
                DeckCardAction::TransmuteAllCopies => "Transmute",
                DeckCardAction::UpgradeAllCopies => "Upgrade",
                DeckCardAction::RemoveOne => "Remove",
            })
            .layout(
                Layout::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::Bottom, 0.px())
                    .position(Edge::Left, 50.pct())
                    .translate((-50).pct(), 50.pct()),
            )
            .button_type(ButtonType::Secondary)
        })
    }

    fn deck_card_effect(
        &self,
        variant: CardVariant,
        quantity: Option<u32>,
    ) -> Option<&DeckCardEffect> {
        if let Some(filter) = self.filter {
            if !card_filter::matches(filter, variant) {
                return None;
            }
        }

        if let Some(effect) = self.effect.as_ref() {
            match effect.action {
                DeckCardAction::DuplicateTo3Copies if quantity.unwrap_or_default() >= 3 => None,
                _ => Some(effect),
            }
        } else {
            None
        }
    }

    fn title(&self) -> impl Component {
        let (title, message) = if let Some(effect) = &self.effect {
            match effect.action {
                DeckCardAction::DuplicateTo3Copies => {
                    ("Duplication", Some("Pick a card to have three copies of."))
                }
                DeckCardAction::TransmuteAllCopies => (
                    "Transmutation",
                    Some(
                        "Pick a card to transform all copies into a random card of a higher rarity.",
                    ),
                ),
                DeckCardAction::UpgradeAllCopies => {
                    ("Upgrade", Some("Pick a card to upgrade all copies."))
                }
                DeckCardAction::RemoveOne => {
                    ("Removal", Some("Pick a card to remove one copy from your deck."))
                }
            }
        } else {
            ("Deck", None)
        };

        Column::new("DeckEditorTitle")
            .style(Style::new().margin(Edge::Bottom, 32.px()))
            .child(Text::new(title).font_size(FontSize::PanelTitle))
            .child(message.map(|m| {
                Text::new(
                    if let Some(times) = self.effect.and_then(|e| (e.times > 1).then_some(e.times))
                    {
                        format!("{} Choices remaining: {}.", m, times)
                    } else {
                        m.to_string()
                    },
                )
                .font_size(FontSize::Body)
            }))
    }
}
impl<'a> Panel for DeckEditorPanel<'a> {
    fn address(&self) -> PanelAddress {
        self.address
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
        let deck = &self.player.adventure.as_ref()?.deck;
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
                    .child(self.title())
                    .child(
                        self.card_section(
                            deck.identities
                                .iter()
                                .chain(deck.sigils.iter())
                                .map(|variant| (variant, None))
                                .chain(
                                    deck.cards
                                        .iter()
                                        .map(|(variant, quantity)| (variant, Some(quantity))),
                                )
                                .collect(),
                        ),
                    ),
            )
            .build()
    }
}
