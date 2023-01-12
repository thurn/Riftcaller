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

use std::collections::HashMap;

use core_ui::action_builder::ActionBuilder;
use core_ui::animations::{
    self, default_duration, AnimateToElement, CreateTargetAtIndex, DestroyElement,
    InterfaceAnimation,
};
use core_ui::conditional::Conditional;
use core_ui::design::{BackgroundColor, FontSize};
use core_ui::draggable::Draggable;
use core_ui::drop_target::DropTarget;
use core_ui::prelude::*;
use core_ui::style::Corner;
use core_ui::text::Text;
use data::card_name::CardName;
use data::deck::Deck;
use data::player_data::PlayerData;
use data::primitives::Side;
use data::user_actions::DeckEditorAction;
use deck_card::deck_card_slot::DeckCardSlot;
use deck_card::{CardHeight, DeckCard};
use element_names::{CurrentDraggable, TargetName};
use panel_address::CollectionBrowserFilters;
use protos::spelldawn::{FlexAlign, FlexDirection, FlexJustify, FlexPosition};

use crate::card_list;
use crate::card_list_card_name::CardListCardName;

/// Returns an iterator over cards in 'collection' which match a given
/// [CollectionBrowserFilters]
pub fn get_matching_cards(
    collection: &HashMap<CardName, u32>,
    _: CollectionBrowserFilters,
) -> impl Iterator<Item = (CardName, u32)> + '_ {
    collection
        .iter()
        .map(|(card_name, count)| (*card_name, *count))
        .filter(|(name, _)| rules::get(*name).side == Side::Champion)
}

pub struct CollectionBrowser<'a> {
    pub player: &'a PlayerData,
    pub deck: &'a Deck,
    pub collection: &'a HashMap<CardName, u32>,
    pub filters: CollectionBrowserFilters,
}

impl<'a> CollectionBrowser<'a> {
    fn card_row(&self, cards: Vec<&(CardName, u32)>) -> impl Component {
        let empty_slots = if cards.len() < 4 { 4 - cards.len() } else { 0 };
        Row::new("CardRow")
            .style(
                Style::new()
                    .flex_grow(1.0)
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center),
            )
            .child_nodes(cards.into_iter().map(|(n, quantity)| self.collection_card(*n, *quantity)))
            .children((0..empty_slots).map(|_| {
                DeckCardSlot::new(CardHeight::vh(36.0))
                    .layout(Layout::new().margin(Edge::All, 4.px()))
            }))
    }

    fn collection_card(&self, card_name: CardName, quantity: u32) -> Option<Node> {
        let in_deck = quantity == *self.deck.cards.get(&card_name).unwrap_or(&0);

        let slot = DeckCardSlot::new(CardHeight::vh(36.0))
            .layout(Layout::new().margin(Edge::All, 16.px()))
            .card(Some(DeckCard::new(card_name).quantity(quantity).draggable((!in_deck).then(
                || {
                    Draggable::new(card_name.to_string())
                        .drop_target(element_names::CARD_LIST)
                        .over_target_indicator(move || CardListCardName::new(card_name).build())
                        .on_drop(Some(self.drop_action(card_name)))
                        .hide_indicator_children(vec![element_names::deck_card_quantity(card_name)])
                },
            ))));

        if in_deck {
            Column::new(element_names::deck_card_slot_overlay(card_name))
                .style(
                    Style::new()
                        .justify_content(FlexJustify::Center)
                        .align_items(FlexAlign::Center),
                )
                .child(slot)
                .child(
                    Row::new("InDeck")
                        .style(
                            Style::new()
                                .position_type(FlexPosition::Absolute)
                                .position(Edge::Left, 50.pct())
                                .position(Edge::Bottom, 0.pct())
                                .translate((-50).pct(), (-50).pct())
                                .background_color(BackgroundColor::TilePanelOverlay)
                                .padding(Edge::All, 8.px())
                                .border_radius(Corner::All, 8.px())
                                .justify_content(FlexJustify::Center)
                                .align_items(FlexAlign::Center),
                        )
                        .child(Text::new("In Deck").font_size(FontSize::Body)),
                )
                .build()
        } else {
            slot.build()
        }
    }

    fn drop_action(&self, name: CardName) -> ActionBuilder {
        let element_name = element_names::card_list_card_name(name);
        let target_name = TargetName(element_name);
        ActionBuilder::new().action(DeckEditorAction::AddToDeck(name)).update(
            Conditional::if_exists(element_name)
                .then(
                    InterfaceAnimation::new()
                        .start(CurrentDraggable, AnimateToElement::new(element_name))
                        .insert(animations::default_duration(), CurrentDraggable, DestroyElement),
                )
                .or_else(
                    InterfaceAnimation::new()
                        .start(
                            CurrentDraggable,
                            CreateTargetAtIndex::parent(element_names::CARD_LIST)
                                .index(card_list::position_for_card(self.deck, name) as u32)
                                .name(target_name),
                        )
                        .start(
                            CurrentDraggable,
                            // We need to offset this animation because the
                            // target is moving *to* its size while the card is
                            // moving to the target.
                            AnimateToElement::new(target_name).disable_height_half_offset(true),
                        )
                        .insert(default_duration(), CurrentDraggable, DestroyElement),
                ),
        )
    }

    fn sort_cards(&self, cards: &mut [(CardName, u32)]) {
        cards.sort_by_key(|(name, _)| {
            let definition = rules::get(*name);
            let cost = definition.cost.mana.unwrap_or_default();
            (definition.side, definition.school, definition.card_type, cost, name.displayed_name())
        });
    }
}

impl<'a> Component for CollectionBrowser<'a> {
    fn build(self) -> Option<Node> {
        let mut cards = get_matching_cards(self.collection, self.filters).collect::<Vec<_>>();
        self.sort_cards(&mut cards);
        let row_one = cards.iter().skip(self.filters.offset).take(4).collect::<Vec<_>>();
        let row_two = cards.iter().skip(self.filters.offset + 4).take(4).collect::<Vec<_>>();
        DropTarget::new(element_names::COLLECTION_BROWSER)
            .style(
                Style::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_grow(1.0)
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center),
            )
            .child(self.card_row(row_one))
            .child(self.card_row(row_two))
            .build()
    }
}
