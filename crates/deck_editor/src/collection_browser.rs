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

use core_ui::actions;
use core_ui::design::BLACK;
use core_ui::drop_target::DropTarget;
use core_ui::prelude::*;
use data::card_name::CardName;
use data::deck::Deck;
use data::player_name::PlayerId;
use data::user_actions::{DeckEditorAction, UserAction};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::update_interface_element_command::InterfaceUpdate;
use protos::spelldawn::{
    AnimateToChildIndex, AnimateToElementPosition, EasingMode, FlexAlign, FlexDirection,
    FlexJustify, StandardAction, TimeValue, UpdateInterfaceElementCommand,
};

use crate::card_list;
use crate::deck_card::DeckCard;

#[allow(dead_code)]
#[derive(Debug)]
pub struct CollectionBrowser<'a> {
    player_id: PlayerId,
    open_deck: Option<&'a Deck>,
}

impl<'a> CollectionBrowser<'a> {
    pub fn new(player_id: PlayerId, open_deck: Option<&'a Deck>) -> Self {
        Self { player_id, open_deck }
    }
}

fn card_row(number: u32, cards: impl Iterator<Item = DeckCard>) -> impl Component {
    Row::new(format!("CardRow{number}"))
        .style(
            Style::new()
                .flex_grow(1.0)
                .align_items(FlexAlign::Center)
                .justify_content(FlexJustify::Center),
        )
        .children(cards)
}

impl<'a> Component for CollectionBrowser<'a> {
    fn build(self) -> Option<Node> {
        let row_one = vec![
            CardName::TestOverlordSpell,
            CardName::TestOverlordIdentity,
            CardName::TestScheme31,
            CardName::TestMinionDealDamage,
        ];
        let row_two = vec![
            CardName::TestMortalMinion,
            CardName::TestAbyssalMinion,
            CardName::TestInfernalMinion,
            CardName::TestProject2Cost,
        ];

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
            .child(card_row(
                1,
                row_one.into_iter().map(|card_name| {
                    DeckCard::new(card_name)
                        .layout(Layout::new().margin(Edge::All, 16.px()))
                        .on_drop(self.open_deck.map(|deck| drop_action(card_name, deck)))
                }),
            ))
            .child(card_row(
                2,
                row_two.into_iter().map(|card_name| {
                    DeckCard::new(card_name)
                        .layout(Layout::new().margin(Edge::All, 16.px()))
                        .on_drop(self.open_deck.map(|deck| drop_action(card_name, deck)))
                }),
            ))
            .build()
    }
}

fn drop_action(name: CardName, open_deck: &Deck) -> StandardAction {
    let update = if open_deck.cards.contains_key(&name) {
        InterfaceUpdate::AnimateToElementPosition(AnimateToElementPosition {
            target_element_name: format!("{}Title", name),
            duration: Some(TimeValue { milliseconds: 300 }),
            easing: EasingMode::Linear.into(),
        })
    } else {
        InterfaceUpdate::AnimateToChildIndex(AnimateToChildIndex {
            parent_element_name: "DeckCardList".to_string(),
            index: card_list::position_for_card(open_deck, name) as u32,
            duration: Some(TimeValue { milliseconds: 300 }),
            easing: EasingMode::Linear.into(),
        })
    };
    StandardAction {
        payload: actions::payload(UserAction::DeckEditorAction(DeckEditorAction::AddToDeck(
            name,
            open_deck.id,
        ))),
        update: Some(actions::command_list(vec![Command::UpdateInterfaceElement(
            UpdateInterfaceElementCommand {
                element_name: format!("{}Title", name),
                interface_update: Some(update),
            },
        )])),
    }
}
