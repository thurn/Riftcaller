use core_data::game_primitives::{AbilityId, CardId, GameObjectId, HasAbilityId, HasCardId};
use game_data::animation_tracker::{AnimationState, GameAnimation};
use game_data::delegate_data::Scope;
use game_data::game_state::GameState;
use game_data::special_effects::{Projectile, SpecialEffect, TimedEffectData};

use crate::CardDefinitionExt;

/// Whether an alert popup should be shown when [show] is called.
#[derive(Eq, PartialEq)]
pub enum ShowAlert {
    Yes,
    No,
}

/// Display the standard visual effect defined for a card on the provided
/// `target`.
///
/// Shows an alert popup if [ShowAlert::Yes] is passed.
pub fn show(
    game: &mut GameState,
    scope: Scope,
    target: impl Into<GameObjectId>,
    show_alert: ShowAlert,
) {
    if game.animations.state == AnimationState::Track {
        VisualEffects::new()
            .optional_ability_alert((show_alert == ShowAlert::Yes).then_some(scope.ability_id()))
            .optional_timed_effect(
                target,
                game.card(scope).definition().config.visual_effect.clone(),
            )
            .apply(game);
    }
}

/// Shows an alert popup for the [Scope] ability.
pub fn show_alert(game: &mut GameState, scope: Scope) {
    if game.animations.state == AnimationState::Track {
        VisualEffects::new().ability_alert(scope.ability_id()).apply(game);
    }
}

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

    /// Optional version of [Self::timed_effect].
    pub fn optional_timed_effect(
        self,
        target: impl Into<GameObjectId>,
        effect: Option<TimedEffectData>,
    ) -> Self {
        if let Some(e) = effect {
            self.timed_effect(target, e)
        } else {
            self
        }
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
