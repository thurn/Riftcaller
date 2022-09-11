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
use core_ui::prelude::*;
use data::card_name::CardName;
use data::player_name::PlayerId;
use data::primitives::DeckId;
use data::user_actions::{DeckEditorAction, UserAction};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::update_interface_element_command::InterfaceUpdate;
use protos::spelldawn::{
    AnimateToChildIndex, EasingMode, FlexAlign, FlexJustify, StandardAction, TimeValue,
    UpdateInterfaceElementCommand,
};

use crate::ui_card::UICard;

#[derive(Debug)]
pub struct CollectionBrowser {
    player_id: PlayerId,
}

impl CollectionBrowser {
    pub fn new(player_id: PlayerId) -> Self {
        Self { player_id }
    }
}

fn card_row(number: u32, cards: impl Iterator<Item = UICard>) -> impl Component {
    Row::new(format!("CardRow{number}"))
        .style(
            Style::new()
                .flex_grow(1.0)
                .align_items(FlexAlign::Center)
                .justify_content(FlexJustify::Center),
        )
        .children(cards)
}

impl Component for CollectionBrowser {
    fn build(self) -> RenderResult {
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

        Column::new(format!("CollectionBrowser{:?}", self.player_id))
            .style(
                Style::new()
                    .background_color(BLACK)
                    .flex_grow(1.0)
                    .margin(Edge::Horizontal, 112.px())
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center),
            )
            .child(card_row(
                1,
                row_one.into_iter().map(|card_name| {
                    UICard::new(card_name)
                        .layout(Layout::new().margin(Edge::All, 16.px()))
                        .on_drop(drop_action(card_name, DeckId::new(1)))
                }),
            ))
            .child(card_row(
                2,
                row_two.into_iter().map(|card_name| {
                    UICard::new(card_name)
                        .layout(Layout::new().margin(Edge::All, 16.px()))
                        .on_drop(drop_action(card_name, DeckId::new(1)))
                }),
            ))
            .build()
    }
}

fn drop_action(name: CardName, active_deck: DeckId) -> StandardAction {
    StandardAction {
        payload: actions::payload(UserAction::DeckEditorAction(DeckEditorAction::AddToDeck(
            name,
            active_deck,
        ))),
        update: Some(actions::command_list(vec![Command::UpdateInterfaceElement(
            UpdateInterfaceElementCommand {
                element_name: format!("{}Title", name),
                interface_update: Some(InterfaceUpdate::AnimateToChildIndex(AnimateToChildIndex {
                    parent_element_name: "DeckCardList".to_string(),
                    index: 0,
                    duration: Some(TimeValue { milliseconds: 300 }),
                    easing: EasingMode::Linear.into(),
                })),
            },
        )])),
    }
}
