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

use adapters::response_builder::ResponseBuilder;
use core_ui::design::{BackgroundColor, BorderColor, FontColor, FontSize};
use core_ui::prelude::*;
use core_ui::style::Corner;
use core_ui::text::Text;
use game_data::primitives::{Milliseconds, RoomId};
use game_data::tutorial_data::{GameTutorialState, TooltipAnchor, TutorialDisplay};
use protos::spelldawn::show_arrow_bubble::ArrowBubbleAnchor;
use protos::spelldawn::tutorial_effect::TutorialEffectType;
use protos::spelldawn::{
    ArrowBubbleCorner, PlayerName, ShowArrowBubble, ShowToast, TextAlign, TutorialEffect,
    WhiteSpace,
};

pub fn render(builder: &ResponseBuilder, state: &GameTutorialState) -> Vec<TutorialEffect> {
    state
        .display
        .iter()
        .map(|display| TutorialEffect {
            tutorial_effect_type: Some(render_effect(builder, display)),
        })
        .collect()
}

fn render_effect(builder: &ResponseBuilder, display: &TutorialDisplay) -> TutorialEffectType {
    match display {
        TutorialDisplay::Tooltip(tooltip) => TutorialEffectType::ArrowBubble(ShowArrowBubble {
            text: tooltip.text.clone(),
            color: Some(BackgroundColor::Tooltip.into()),
            font_color: Some(FontColor::Tooltip.into()),
            arrow_bubble_anchor: Some(match tooltip.anchor {
                TooltipAnchor::RaidRoom(room) => {
                    ArrowBubbleAnchor::Room(adapters::room_identifier(room))
                }
                TooltipAnchor::GainMana => ArrowBubbleAnchor::PlayerMana(PlayerName::User.into()),
                TooltipAnchor::DrawCard => ArrowBubbleAnchor::PlayerDeck(PlayerName::User.into()),
            }),
            idle_timer: Some(adapters::time_value(tooltip.delay)),
            arrow_corner: if tooltip.anchor == TooltipAnchor::GainMana
                || tooltip.anchor == TooltipAnchor::RaidRoom(RoomId::Vault)
            {
                ArrowBubbleCorner::BottomRight
            } else {
                ArrowBubbleCorner::BottomLeft
            }
            .into(),
            ..ShowArrowBubble::default()
        }),
        TutorialDisplay::SpeechBubble(speech_bubble) => {
            TutorialEffectType::ArrowBubble(ShowArrowBubble {
                text: speech_bubble.text.clone(),
                color: Some(BackgroundColor::SpeechBubble.into()),
                font_color: Some(FontColor::SpeechBubble.into()),
                arrow_bubble_anchor: Some(ArrowBubbleAnchor::Player(
                    builder.to_player_name(speech_bubble.side),
                )),
                idle_timer: Some(adapters::time_value(speech_bubble.delay)),
                hide_time: Some(adapters::time_value(Milliseconds(4000))),
                ..ShowArrowBubble::default()
            })
        }
        TutorialDisplay::Toast(toast) => TutorialEffectType::ShowToast(ShowToast {
            node: make_toast(&toast.text),
            idle_timer: Some(adapters::time_value(toast.delay)),
            hide_time: toast.hide_after.map(adapters::time_value),
        }),
    }
}

fn make_toast(text: &str) -> Option<Node> {
    Row::new("Toast")
        .style(
            Style::new()
                .padding(Edge::Horizontal, 12.px())
                .max_width(400.px())
                .background_color(BackgroundColor::Toast)
                .border_radius(Corner::All, 12.px())
                .border_color(Edge::All, BorderColor::Toast)
                .border_width(Edge::All, 2.px()),
        )
        .child(
            Text::new(text)
                .font_size(FontSize::Toast)
                .color(FontColor::Toast)
                .text_align(TextAlign::MiddleLeft)
                .white_space(WhiteSpace::Normal),
        )
        .build()
}
