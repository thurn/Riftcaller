// Copyright © Riftcaller 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core_data::game_primitives::{AbilityId, CardId, GameObjectId, HasAbilityId, HasCardId};
use game_data::animation_tracker::GameAnimation;
use game_data::game_state::GameState;
use game_data::special_effects::{Projectile, SpecialEffect, TimedEffectData};

#[derive(Clone, Debug, Default)]
pub struct VisualEffects {
    ability_triggered: Option<AbilityId>,
    effects: Vec<SpecialEffect>,
}

impl VisualEffects {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(self, game: &mut GameState) {
        if let Some(id) = self.ability_triggered {
            game.add_animation(|| GameAnimation::AbilityTriggered(id, self.effects));
        } else {
            game.add_animation(|| GameAnimation::CustomEffects(self.effects));
        }
    }

    /// Pushes a [GameAnimation] indicating the ability represented by the
    /// provided ability ID should have a trigger animation shown in the UI.
    pub fn ability_alert(mut self, ability_id: impl HasAbilityId) -> Self {
        self.ability_triggered = Some(ability_id.ability_id());
        self
    }

    /// Equivalent function to [Self::ability_alert] which triggers if the
    /// provided id is present.
    pub fn optional_ability_alert(mut self, ability_id: Option<AbilityId>) -> Self {
        self.ability_triggered = ability_id;
        self
    }

    /// Shows an `alert` if the provided `number` is not zero.
    pub fn ability_alert_if_nonzero(self, ability_id: impl HasAbilityId, number: u32) -> Self {
        if number > 0 {
            self.ability_alert(ability_id)
        } else {
            self
        }
    }

    /// Creates a [SpecialEffect::TimedEffect] playing a visual effect on the
    /// indicated target.
    pub fn timed_effect(
        mut self,
        target: impl Into<GameObjectId>,
        effect: TimedEffectData,
    ) -> Self {
        self.effects.push(SpecialEffect::TimedEffect { target: target.into(), effect });
        self
    }

    /// Creates a [SpecialEffect::CardMovementEffect] playing an effect when the
    /// indicated card is moved to a new position.
    pub fn card_movement_effect(mut self, asset: Projectile, card_id: impl HasCardId) -> Self {
        self.effects
            .push(SpecialEffect::CardMovementEffect { card_id: card_id.card_id(), effect: asset });
        self
    }

    /// Applies a series of [Self::card_movement_effect] effects.
    pub fn card_movement_effects(mut self, asset: Projectile, cards: &[CardId]) -> Self {
        self.effects.extend(
            cards
                .iter()
                .map(|id| SpecialEffect::CardMovementEffect { card_id: *id, effect: asset }),
        );
        self
    }
}
