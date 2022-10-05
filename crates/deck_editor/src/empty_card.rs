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

use core_ui::design::ORANGE_900;
use core_ui::prelude::*;

use crate::deck_card::{CARD_ASPECT_RATIO, CARD_HEIGHT};

/// Represents an empty slot in the collection browser to fill space.
#[derive(Clone)]
pub struct EmptyCard {}

impl Component for EmptyCard {
    fn build(self) -> Option<Node> {
        Column::new("EmptyCard")
            .style(
                Style::new()
                    .background_color(ORANGE_900)
                    .width((CARD_HEIGHT * CARD_ASPECT_RATIO).vh())
                    .height(CARD_HEIGHT.vh())
                    .margin(Edge::All, 16.px()),
            )
            .build()
    }
}
