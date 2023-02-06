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

use core_ui::prelude::Node;
use game_data::card_definition::{Ability, CardDefinition};
use game_data::primitives::AbilityIndex;
use game_data::text::RulesTextContext;
use game_data::text2::Text2;
use protos::spelldawn::RulesText;

/// Primary function which turns the current state of a card into its client
/// [RulesText] representation
pub fn build(context: &RulesTextContext, definition: &CardDefinition) -> RulesText {
    RulesText::default()
}

/// Builds the rules text for a single [Ability], not including its cost (if
/// any).
pub fn ability_text(context: &RulesTextContext, ability: &Ability) -> String {
    String::new()
}

/// Builds the supplemental info display for a card, which displays additional
/// help information and appears on long-press.
///
/// If an `ability_index` is provided, only supplemental info for that index is
/// returned. Otherwise, supplemental info for all abilities is returned.
pub fn build_supplemental_info(
    context: &RulesTextContext,
    ability_index: Option<AbilityIndex>,
) -> Option<Node> {
    None
}

fn buid_paragraph(text: Vec<Text2>) -> String {
    String::new()
}
