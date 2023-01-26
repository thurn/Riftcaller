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
use core_ui::design::{BackgroundColor, FontColor};
use data::tutorial_data::{GameTutorialState, TooltipAnchor, TutorialDisplay};
use protos::spelldawn::show_arrow_bubble::ArrowBubbleAnchor;
use protos::spelldawn::tutorial_effect::TutorialEffectType;
use protos::spelldawn::{PlayerName, ShowArrowBubble, TutorialEffect};

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
        TutorialDisplay::Tooltip(text, anchor) => {
            TutorialEffectType::ArrowBubble(ShowArrowBubble {
                text: text.clone(),
                color: Some(BackgroundColor::TooltipBackground.into()),
                font_color: Some(FontColor::TooltipText.into()),
                arrow_bubble_anchor: Some(match anchor {
                    TooltipAnchor::RaidRoom(room) => {
                        ArrowBubbleAnchor::Room(adapters::room_identifier(*room))
                    }
                    TooltipAnchor::GainMana => {
                        ArrowBubbleAnchor::PlayerMana(PlayerName::User.into())
                    }
                    TooltipAnchor::DrawCard => {
                        ArrowBubbleAnchor::PlayerDeck(PlayerName::User.into())
                    }
                }),
                ..ShowArrowBubble::default()
            })
        }
        TutorialDisplay::SpeechBubble(text, side) => {
            TutorialEffectType::ArrowBubble(ShowArrowBubble {
                text: text.clone(),
                arrow_bubble_anchor: Some(ArrowBubbleAnchor::Player(builder.to_player_name(*side))),
                ..ShowArrowBubble::default()
            })
        }
    }
}
