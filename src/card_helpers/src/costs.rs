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

use game_data::card_definition::CustomCost;
use game_data::card_state::CardPosition;
use game_data::primitives::AbilityId;
use game_data::text::TextElement;
use rules::mutations;

/// A [CustomCost] which allows an ability to be activated by sacrificing the
/// card.
pub fn sacrifice_cost() -> Option<CustomCost<AbilityId>> {
    Some(CustomCost {
        can_pay: |game, ability_id| {
            game.card(ability_id.card_id).is_face_up()
                && game.card(ability_id.card_id).position().in_play()
        },
        pay: |game, ability_id| {
            game.ability_state_mut(ability_id).turn = Some(game.info.turn);
            mutations::move_card(
                game,
                ability_id.card_id,
                CardPosition::DiscardPile(ability_id.side()),
            )
        },
        description: Some(TextElement::Literal("Sacrifice".to_string())),
    })
}
