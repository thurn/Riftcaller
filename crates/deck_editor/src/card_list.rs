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

use core_ui::design::RED_900;
use core_ui::drop_target::DropTarget;
use core_ui::prelude::*;
use core_ui::{actions, panel};
use data::card_name::CardName;
use data::deck::Deck;
use data::player_data::PlayerData;
use data::primitives::DeckIndex;
use data::user_actions::{DeckEditorAction, UserAction};
use panel_address::{CollectionBrowserFilters, DeckEditorData, PanelAddress};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::update_interface_element_command::InterfaceUpdate;
use protos::spelldawn::{
    AnimateToElementPositionAndDestroy, FlexAlign, FlexDirection, StandardAction, TimeValue,
    UpdateInterfaceElementCommand,
};

use crate::deck_card_title::DeckCardTitle;
use crate::deck_editor_panel::EDITOR_COLUMN_WIDTH;
use crate::deck_tile::DeckTile;
use crate::editor_column_scroll::EditorColumnScroll;

/// Displays the cards contained within a single deck
#[derive(Debug)]
#[allow(dead_code)]
pub struct CardList<'a> {
    player: &'a PlayerData,
    deck: &'a Deck,
}

impl<'a> CardList<'a> {
    pub fn new(player: &'a PlayerData, deck: &'a Deck) -> Self {
        CardList { player, deck }
    }
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
                DropTarget::new("CardList")
                    .style(
                        Style::new()
                            .flex_direction(FlexDirection::Column)
                            .background_color(RED_900)
                            .width(EDITOR_COLUMN_WIDTH.vw())
                            .align_items(FlexAlign::Center)
                            .padding(Edge::All, 1.vw()),
                    )
                    .child(DeckTile::new(self.deck).action(panel::set(PanelAddress::DeckEditor(
                        DeckEditorData {
                            deck: Some(self.deck.index),
                            show_edit_options: true,
                            collection_filters: CollectionBrowserFilters::default(),
                        },
                    ))))
                    .children(sorted_deck(self.deck).into_iter().map(|(card_name, count)| {
                        DeckCardTitle::new(*card_name)
                            .count(*count)
                            .on_drop(Some(drop_action(*card_name, self.deck.index)))
                    })),
            )
            .build()
    }
}

fn drop_action(name: CardName, active_deck: DeckIndex) -> StandardAction {
    StandardAction {
        payload: actions::payload(UserAction::DeckEditorAction(DeckEditorAction::RemoveFromDeck(
            name,
            active_deck,
        ))),
        update: Some(actions::command_list(vec![Command::UpdateInterfaceElement(
            UpdateInterfaceElementCommand {
                element_name: "<OverTargetIndicator>".to_string(),
                interface_update: Some(InterfaceUpdate::AnimateToElementPosition(
                    AnimateToElementPositionAndDestroy {
                        target_element_name: name.to_string(),
                        fallback_target_element_name: "CollectionBrowser".to_string(),
                        duration: Some(TimeValue { milliseconds: 300 }),
                    },
                )),
            },
        )])),
        request_fields: HashMap::new(),
    }
}
