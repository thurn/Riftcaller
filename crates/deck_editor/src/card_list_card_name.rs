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
use core_ui::design::{BackgroundColor, Font, FontColor, FontSize, OVERLAY_BORDER};
use core_ui::draggable::Draggable;
use core_ui::prelude::*;
use core_ui::style::Corner;
use core_ui::text::Text;
use data::card_name::CardName;
use protos::spelldawn::{
    BackgroundImageAutoSize, FlexAlign, FlexDirection, FlexJustify, FlexPosition, ImageScaleMode,
    StandardAction,
};

use crate::deck_editor_panel::EDITOR_COLUMN_WIDTH;

#[derive(Debug)]
pub struct CardListCardName {
    layout: Layout,
    card_name: CardName,
    on_drop: Option<StandardAction>,
    count: Option<u32>,
}

impl CardListCardName {
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

impl Component for CardListCardName {
    fn build(self) -> Option<Node> {
        let definition = rules::get(self.card_name);
        let cost = match (definition.cost.mana, definition.config.stats.scheme_points) {
            (Some(mana), _) => Some((mana.to_string(), CardIconType::Mana)),
            (_, Some(scheme_points)) => {
                Some((scheme_points.level_requirement.to_string(), CardIconType::LevelRequirement))
            }
            _ => None,
        };

        Draggable::new(element_names::card_list_card_name(self.card_name))
            .drop_targets(vec!["CollectionBrowser"])
            // .over_target_indicator(move || DeckEditorCard::new(self.card_name).build())
            .on_drop(self.on_drop)
            .horizontal_drag_start_distance(100)
            .remove_original(if let Some(v) = self.count { v < 2 } else { false })
            .style(
                Style::new()
                    .height(72.px())
                    .width((EDITOR_COLUMN_WIDTH - 1).vw())
                    .flex_grow(1.0)
                    .flex_direction(FlexDirection::Row)
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center),
            )
            .child(
                // Image is offset from parent in order to expand touch target
                Column::new("Image").style(
                    Style::new()
                        .position_type(FlexPosition::Absolute)
                        .position(Edge::All, 4.px())
                        .background_image(adapters::sprite(&definition.image))
                        .background_image_scale_mode(ImageScaleMode::ScaleAndCrop)
                        .border_radius(Corner::All, 8.px()),
                ),
            )
            .child(
                Column::new("ImageOverlay").style(
                    Style::new()
                        .position_type(FlexPosition::Absolute)
                        .position(Edge::All, 4.px())
                        .background_color(BackgroundColor::DeckCardNameOverlay)
                        .border_radius(Corner::All, 8.px())
                        .border_color(Edge::All, OVERLAY_BORDER)
                        .border_width(Edge::All, 2.px()),
                ),
            )
            .child(cost.map(|(text, icon)| {
                Column::new("CardCost")
                    .style(
                        Style::new()
                            .height(44.px())
                            .margin(Edge::Vertical, 8.px())
                            .margin(Edge::Left, 12.px())
                            .margin(Edge::Right, 4.px())
                            .flex_shrink(0.0)
                            .background_image(assets::card_icon(icon))
                            .background_image_auto_size(BackgroundImageAutoSize::FromHeight)
                            .justify_content(FlexJustify::Center)
                            .align_items(FlexAlign::Center),
                    )
                    .child(
                        Text::new(text)
                            .font_size(FontSize::CardCost)
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
                        Text::new(self.card_name.displayed_name())
                            .font_size(FontSize::CardName)
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
                            .margin(Edge::Left, 4.px())
                            .margin(Edge::Right, 12.px())
                            .width(40.px())
                            .height(32.px())
                            .border_radius(Corner::All, 8.px()),
                    )
                    .child(Text::new(format!("{}x", c)).font_size(FontSize::CardCount))
            }))
            .build()
    }
}
