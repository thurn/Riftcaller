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

use std::iter;

use core_ui::action_builder::ActionBuilder;
use core_ui::design::BLACK;
use core_ui::drop_target::DropTarget;
use core_ui::prelude::*;
use data::card_name::CardName;
use data::deck::Deck;
use data::player_data::PlayerData;
use data::user_actions::OldDeckEditorAction;
use panel_address::CollectionBrowserFilters;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::update_interface_element_command::InterfaceUpdate;
use protos::spelldawn::{
    AnimateDraggableToChildIndex, AnimateToElementPositionAndDestroy, FlexAlign, FlexDirection,
    FlexJustify, StandardAction, TimeValue, UpdateInterfaceElementCommand,
};

use crate::card_list;
use crate::deck_editor_card::DeckEditorCard;
use crate::empty_card::EmptyCard;

/// Returns an iterator over cards owned by 'player' which match a given
/// [CollectionBrowserFilters]
pub fn get_matching_cards(
    player: &PlayerData,
    _: CollectionBrowserFilters,
) -> impl Iterator<Item = (CardName, u32)> + '_ {
    player.collection.iter().map(|(card_name, count)| (*card_name, *count))
}

pub struct CollectionBrowser<'a> {
    pub player: &'a PlayerData,
    pub open_deck: Option<&'a Deck>,
    pub filters: CollectionBrowserFilters,
}

fn card_row(open_deck: Option<&Deck>, cards: Vec<&(CardName, u32)>) -> impl Component {
    let empty_slots = if cards.len() < 4 { 4 - cards.len() } else { 0 };
    Row::new("CardRow")
        .style(
            Style::new()
                .flex_grow(1.0)
                .align_items(FlexAlign::Center)
                .justify_content(FlexJustify::Center),
        )
        .children(cards.into_iter().map(|(name, _)| {
            DeckEditorCard::new(*name)
                .layout(Layout::new().margin(Edge::All, 16.px()))
                .on_drop(open_deck.map(|deck| drop_action(*name, deck)))
        }))
        .children(iter::repeat(EmptyCard {}).take(empty_slots))
}

fn sort_cards(cards: &mut [(CardName, u32)]) {
    cards.sort_by_key(|(name, _)| {
        let definition = rules::get(*name);
        let cost = definition.cost.mana.unwrap_or_default();
        (definition.side, definition.school, definition.card_type, cost, name.displayed_name())
    });
}

impl<'a> Component for CollectionBrowser<'a> {
    fn build(self) -> Option<Node> {
        let mut cards = get_matching_cards(self.player, self.filters).collect::<Vec<_>>();
        sort_cards(&mut cards);
        let row_one = cards.iter().skip(self.filters.offset).take(4).collect::<Vec<_>>();
        let row_two = cards.iter().skip(self.filters.offset + 4).take(4).collect::<Vec<_>>();
        DropTarget::new("CollectionBrowser".to_string())
            .style(
                Style::new()
                    .background_color(BLACK)
                    .flex_direction(FlexDirection::Column)
                    .flex_grow(1.0)
                    .margin(Edge::Horizontal, 112.px())
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center),
            )
            .child(card_row(self.open_deck, row_one))
            .child(card_row(self.open_deck, row_two))
            .build()
    }
}

fn drop_action(name: CardName, open_deck: &Deck) -> StandardAction {
    let update = if open_deck.cards.contains_key(&name) {
        InterfaceUpdate::AnimateToElementPosition(AnimateToElementPositionAndDestroy {
            target_element_name: format!("{}Title", name),
            fallback_target_element_name: "".to_string(),
            animation: None,
        })
    } else {
        InterfaceUpdate::AnimateToChildIndex(AnimateDraggableToChildIndex {
            parent_element_name: "CardList".to_string(),
            index: card_list::position_for_card(open_deck, name) as u32,
            duration: Some(TimeValue { milliseconds: 300 }),
        })
    };

    ActionBuilder::new()
        .update(Command::UpdateInterfaceElement(UpdateInterfaceElementCommand {
            element_name: "<OverTargetIndicator>".to_string(),
            interface_update: Some(update),
        }))
        .action(OldDeckEditorAction::AddToDeck(name, open_deck.index))
        .build()
}
