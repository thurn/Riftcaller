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

use game_data::card_state::CardState;
use game_data::primitives::CardType;

pub type CardPredicate = fn(&&CardState) -> bool;

pub fn is_type(card: &&CardState, card_type: CardType) -> bool {
    rules::get(card.variant).card_type == card_type
}

pub fn artifact(card: &&CardState) -> bool {
    is_type(card, CardType::Artifact)
}
