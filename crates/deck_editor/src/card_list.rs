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
use core_ui::design::RED_900;
use core_ui::drop_target::DropTarget;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use data::card_name::CardName;
use data::deck::Deck;
use data::player_data::PlayerData;
use data::primitives::DeckId;
use data::user_actions::{DeckEditorAction, UserAction};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::update_interface_element_command::InterfaceUpdate;
use protos::spelldawn::{
    AnimateToElementPosition, EasingMode, FlexAlign, FlexDirection, ScrollBarVisibility,
    StandardAction, TimeValue, TouchScrollBehavior, UpdateInterfaceElementCommand,
};

use crate::deck_card_title::DeckCardTitle;
use crate::deck_editor_panel::EDITOR_COLUMN_WIDTH;

/// Displays the cards contained within a single deck
#[allow(dead_code)]
#[derive(Debug)]
pub struct CardList<'a> {
    player: &'a PlayerData,
    deck: &'a Deck,
}

impl<'a> CardList<'a> {
    pub fn new(player: &'a PlayerData, deck: &'a Deck) -> Self {
        CardList { player, deck }
    }
}

impl<'a> Component for CardList<'a> {
    fn build(self) -> Option<Node> {
        DropTarget::new("DeckCardList")
            .style(
                Style::new()
                    .flex_direction(FlexDirection::Column)
                    .background_color(RED_900)
                    .width(EDITOR_COLUMN_WIDTH.vw())
                    .align_items(FlexAlign::Center)
                    .padding(Edge::All, 1.vw()),
            )
            .child(
                ScrollView::new("DeckCardListScroll")
                    .vertical_scrollbar_visibility(ScrollBarVisibility::Hidden)
                    .horizontal_scrollbar_visibility(ScrollBarVisibility::Hidden)
                    .touch_scroll_behavior(TouchScrollBehavior::Clamped)
                    .scroll_deceleration_rate(0.0)
                    .children(self.deck.cards.iter().map(|(card_name, count)| {
                        DeckCardTitle::new(*card_name)
                            .count(*count)
                            .on_drop(Some(drop_action(*card_name, self.deck.id)))
                    })),
            )
            .build()
    }
}

fn drop_action(name: CardName, active_deck: DeckId) -> StandardAction {
    StandardAction {
        payload: actions::payload(UserAction::DeckEditorAction(DeckEditorAction::RemoveFromDeck(
            name,
            active_deck,
        ))),
        update: Some(actions::command_list(vec![Command::UpdateInterfaceElement(
            UpdateInterfaceElementCommand {
                element_name: format!("{}Title", name),
                interface_update: Some(InterfaceUpdate::AnimateToElementPosition(
                    AnimateToElementPosition {
                        target_element_name: name.to_string(),
                        duration: Some(TimeValue { milliseconds: 300 }),
                        easing: EasingMode::Linear.into(),
                    },
                )),
            },
        )])),
    }
}
