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
use game_data::delegate_data::RaidOutcome;
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;

use crate::mana::ManaPurpose;
use crate::{mana, mutations};

pub fn handle(game: &mut GameState, effect: GameEffect) -> Result<()> {
    match effect {
        GameEffect::AbortCurrentGameAction => mutations::abort_current_game_action(game),
        GameEffect::SacrificeCard(card_id) => mutations::sacrifice_card(game, card_id)?,
        GameEffect::LoseMana(side, amount) => {
            mana::spend(game, side, ManaPurpose::PayForTriggeredAbility, amount)?
        }
        GameEffect::LoseActions(side, amount) => {
            mutations::spend_action_points(game, side, amount)?
        }
        GameEffect::EndRaid => mutations::end_raid(game, RaidOutcome::Failure)?,
        GameEffect::TakeDamage(ability_id, amount) => {
            mutations::deal_damage(game, ability_id, amount)?
        }
    }

    Ok(())
}
