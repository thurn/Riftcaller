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

use core_ui::action_builder::ActionBuilder;
use core_ui::animations::{
    self, AnimateStyle, AnimateToElement, DestroyElement, InterfaceAnimation,
};
use core_ui::conditional::Conditional;
use core_ui::drop_target::DropTarget;
use core_ui::prelude::*;
use element_names::CurrentDraggable;
use game_data::card_name::CardName;
use game_data::deck::Deck;
use game_data::user_actions::DeckEditorAction;
use protos::spelldawn::animate_element_style::Property;
use protos::spelldawn::{FlexAlign, FlexDirection, FlexVector2};

use crate::card_list_card_name::CardListCardName;
use crate::deck_editor_panel::EDITOR_COLUMN_WIDTH;
use crate::editor_column_scroll::EditorColumnScroll;

/// Displays the cards contained within a single deck
#[derive(Debug)]
pub struct CardList<'a> {
    pub deck: &'a Deck,
}

/// Standard sorted display order for a deck.
pub fn sorted_deck(deck: &Deck) -> Vec<(&CardName, &u32)> {
    let mut cards = deck.cards.iter().collect::<Vec<_>>();
    sort_cards(&mut cards);
    cards
}

/// Returns the sort position 'card_name' would occupy in 'deck'.
pub fn position_for_card(deck: &Deck, card_name: CardName) -> usize {
    let mut cards = deck.cards.iter().collect::<Vec<_>>();
    if !deck.cards.contains_key(&card_name) {
        cards.push((&card_name, &1));
    }
    sort_cards(&mut cards);
    cards.iter().position(|(n, _)| **n == card_name).expect("card position")
}

fn sort_cards(cards: &mut [(&CardName, &u32)]) {
    cards.sort_by_key(|(name, _)| {
        let definition = rules::get(**name);
        let cost = definition.cost.mana.unwrap_or_default();
        (definition.card_type, cost, name.displayed_name())
    });
}

impl<'a> Component for CardList<'a> {
    fn build(self) -> Option<Node> {
        EditorColumnScroll::new()
            .child(
                DropTarget::new(element_names::CARD_LIST)
                    .style(
                        Style::new()
                            .flex_direction(FlexDirection::Column)
                            .width(EDITOR_COLUMN_WIDTH.vw())
                            .min_height(70.vh())
                            .align_items(FlexAlign::Center)
                            .padding(Edge::All, 1.vw()),
                    )
                    .children(sorted_deck(self.deck).into_iter().map(|(card_name, count)| {
                        CardListCardName::new(*card_name)
                            .count(*count)
                            .on_drop(Some(drop_action(*card_name)))
                    })),
            )
            .build()
    }
}

fn drop_action(name: CardName) -> ActionBuilder {
    ActionBuilder::new().action(DeckEditorAction::RemoveFromDeck(name)).update(
        Conditional::if_exists(element_names::deck_card(name))
            .then(
                InterfaceAnimation::new()
                    .start(CurrentDraggable, AnimateToElement::new(element_names::deck_card(name)))
                    .insert(animations::default_duration(), CurrentDraggable, DestroyElement),
            )
            .or_else(
                InterfaceAnimation::new()
                    .start(
                        CurrentDraggable,
                        AnimateStyle::new(Property::Scale(FlexVector2 { x: 0.1, y: 0.1 })),
                    )
                    .start(
                        CurrentDraggable,
                        AnimateToElement::new(element_names::COLLECTION_BROWSER),
                    )
                    .insert(animations::default_duration(), CurrentDraggable, DestroyElement),
            ),
    )
}
