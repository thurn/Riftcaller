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

use assets;
use assets::CardIconType;
use core_ui::design::{BackgroundColor, Font, FontColor, FontSize, PINK_900};
use core_ui::draggable::Draggable;
use core_ui::prelude::*;
use core_ui::style::Corner;
use core_ui::text::Text;
use data::card_name::CardName;
use protos::spelldawn::{FlexAlign, FlexDirection, FlexJustify, StandardAction};

use crate::deck_editor_card::DeckEditorCard;
use crate::deck_editor_panel::EDITOR_COLUMN_WIDTH;

#[derive(Debug)]
pub struct DeckEditorCardTitle {
    layout: Layout,
    card_name: CardName,
    on_drop: Option<StandardAction>,
    count: Option<u32>,
}

impl DeckEditorCardTitle {
    pub fn new(card_name: CardName) -> Self {
        Self { card_name, layout: Layout::default(), on_drop: None, count: None }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn on_drop(mut self, on_drop: Option<StandardAction>) -> Self {
        self.on_drop = on_drop;
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }
}

impl Component for DeckEditorCardTitle {
    fn build(self) -> Option<Node> {
        let cost = match (
            rules::get(self.card_name).cost.mana,
            rules::get(self.card_name).config.stats.scheme_points,
        ) {
            (Some(mana), _) => Some((mana.to_string(), CardIconType::Mana)),
            (_, Some(scheme_points)) => {
                Some((scheme_points.level_requirement.to_string(), CardIconType::LevelRequirement))
            }
            _ => None,
        };

        Draggable::new(format!("{}Title", self.card_name))
            .drop_targets(vec!["CollectionBrowser"])
            .over_target_indicator(move || DeckEditorCard::new(self.card_name).build())
            .on_drop(self.on_drop)
            .horizontal_drag_start_distance(100)
            .remove_original(if let Some(v) = self.count { v < 2 } else { false })
            .style(
                Style::new()
                    .height(88.px())
                    .width((EDITOR_COLUMN_WIDTH - 1).vw())
                    .flex_grow(1.0)
                    .flex_direction(FlexDirection::Row)
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center)
                    .background_color(PINK_900)
                    .margin(Edge::Vertical, 8.px()),
            )
            .child(cost.map(|(text, icon)| {
                Column::new("CardCost")
                    .style(
                        Style::new()
                            .width(44.px())
                            .height(44.px())
                            .margin(Edge::All, 8.px())
                            .flex_shrink(0.0)
                            .background_image(assets::card_icon(icon))
                            .justify_content(FlexJustify::Center)
                            .align_items(FlexAlign::Center),
                    )
                    .child(
                        Text::new(text, FontSize::CardCost)
                            .layout(Layout::new().margin(Edge::All, 0.px()))
                            .font(Font::CardIcon)
                            .outline_width(1.px())
                            .color(FontColor::CardCost),
                    )
            }))
            .child(
                Column::new("CardTitle")
                    .style(
                        Style::new()
                            .flex_grow(1.0)
                            .justify_content(FlexJustify::Center)
                            .align_items(FlexAlign::FlexStart),
                    )
                    .child(
                        Text::new(self.card_name.displayed_name(), FontSize::CardName)
                            .layout(Layout::new().margin(Edge::All, 0.px())),
                    ),
            )
            .child(self.count.map(|c| {
                Column::new("CardCount")
                    .style(
                        Style::new()
                            .background_color(BackgroundColor::CardCount)
                            .justify_content(FlexJustify::Center)
                            .align_items(FlexAlign::Center)
                            .flex_shrink(0.0)
                            .margin(Edge::All, 8.px())
                            .width(32.px())
                            .height(32.px())
                            .border_radius(Corner::All, 8.px()),
                    )
                    .child(Text::new(c.to_string(), FontSize::CardCount))
            }))
            .build()
    }
}
