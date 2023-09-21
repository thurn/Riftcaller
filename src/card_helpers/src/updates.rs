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

use game_data::game::GameState;
use game_data::game_updates::GameUpdate;
use game_data::primitives::{AbilityId, CardId, GameObjectId, HasAbilityId, HasCardId};
use game_data::special_effects::{Projectile, SpecialEffect, TimedEffectData};

pub struct Updates<'a> {
    game: &'a mut GameState,
    ability_triggered: Option<AbilityId>,
    effects: Vec<SpecialEffect>,
}

impl<'a> Updates<'a> {
    pub fn new(game: &'a mut GameState) -> Self {
        Self { game, ability_triggered: None, effects: vec![] }
    }

    pub fn apply(self) {
        if let Some(id) = self.ability_triggered {
            self.game.record_update(|| GameUpdate::AbilityTriggered(id, self.effects));
        } else {
            self.game.record_update(|| GameUpdate::CustomEffects(self.effects));
        }
    }

    /// Pushes a [GameUpdate] indicating the ability represented by the provided
    /// ability ID should have a trigger animation shown in the UI.    
    pub fn ability_alert(mut self, ability_id: impl HasAbilityId) -> Self {
        self.ability_triggered = Some(ability_id.ability_id());
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
