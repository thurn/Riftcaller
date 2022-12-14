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

//! Renders cards as they're seen in the deck editor and adventure UI

use core_ui::prelude::*;
use data::card_name::CardName;

#[allow(dead_code)]
pub struct DeckCard {
    name: CardName,
    layout: Layout,
}

impl DeckCard {
    pub fn new(name: CardName) -> Self {
        Self { name, layout: Layout::default() }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
}

impl Component for DeckCard {
    fn build(self) -> Option<Node> {
        None
    }
}
