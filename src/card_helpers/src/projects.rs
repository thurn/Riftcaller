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

use anyhow::Result;
use game_data::card_definition::{Ability, CardConfig};
use game_data::delegates::{Delegate, EventDelegate, ProjectTriggeredEvent, Scope};
use game_data::game::GameState;
use game_data::game_actions::GamePrompt;
use game_data::primitives::CardSubtype;
use game_data::text::TextToken;
use rules::{dispatch, flags};

use crate::text;

/// Fires a trigger event for a project or prompts the user to unveil it if it
/// is currently face-down.
pub fn fire_trigger(game: &mut GameState, scope: Scope) -> Result<()> {
    let project_id = scope.card_id();
    if game.card(project_id).is_face_up() && game.card(project_id).position().in_play() {
        dispatch::invoke_event(game, ProjectTriggeredEvent(project_id))
    } else if flags::can_unveil_card(game, project_id) {
        game.player_mut(project_id.side)
            .card_prompt_queue
            .push(GamePrompt::unveil_project(project_id));
        Ok(())
    } else {
        Ok(())
    }
}

pub fn activated_subtype() -> CardConfig {
    CardConfig { subtypes: vec![CardSubtype::Activated], ..CardConfig::default() }
}

pub fn triggered_subtype() -> CardConfig {
    CardConfig { subtypes: vec![CardSubtype::Triggered], ..CardConfig::default() }
}

/// RequirementFn that this delegate's card is currently in play, either face-up
/// or face-down.
pub fn either_face_in_play<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    game.card(scope.card_id()).position().in_play()
}

/// Fires a project trigger at dusk, prompting the player to unveil the card if
/// it is face-down or else immediately invoking its trigger event.
pub fn trigger_at_dusk() -> Delegate {
    Delegate::Dusk(EventDelegate {
        requirement: either_face_in_play,
        mutation: |g, s, _| fire_trigger(g, s),
    })
}

/// Ability to store a fixed amount of mana in a card when it is unveiled.
pub fn store_mana_on_unveil<const N: u32>() -> Ability {
    crate::simple_ability(
        crate::trigger_text(TextToken::Unveil, text![TextToken::StoreMana(N)]),
        crate::when_unveiled(|g, s, _| {
            crate::add_stored_mana(g, s.card_id(), N);
            Ok(())
        }),
    )
}
