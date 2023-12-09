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

use anyhow::Result;
use core_data::game_primitives::{CardId, GameObjectId};
use game_data::delegate_data::Scope;
use game_data::game_state::GameState;
use game_data::prompt_data::{
    BrowserPromptTarget, BrowserPromptValidation, CardSelectorPrompt, GamePrompt, PromptContext,
};
use game_data::special_effects::{Projectile, TimedEffectData};
use rules::visual_effects::VisualEffects;

/// Display a new Card Selector prompt a [CardSelectorPromptBuilder].
///
/// Has no effect if no subject cards have been specified for this selector.
pub fn show(game: &mut GameState, builder: CardSelectorPromptBuilder) -> Result<()> {
    if builder.subjects.is_empty() {
        return Ok(());
    }

    let mut effects = VisualEffects::new();
    let cards = builder.subjects.clone();
    if let Some((id, data)) = builder.visual_effect {
        effects = effects.timed_effect(id, data);
    }

    if builder.show_ability_alert {
        effects = effects.ability_alert(builder.scope);
    }

    if let Some(movement_effects) = builder.movement_effect {
        effects.card_movement_effects(movement_effects, &cards).apply(game);
    }

    game.player_mut(builder.scope.side()).old_prompt_stack.push(GamePrompt::CardSelector(
        CardSelectorPrompt {
            context: builder.context,
            unchosen_subjects: builder.subjects,
            chosen_subjects: vec![],
            target: builder.target,
            validation: builder.validation,
        },
    ));

    Ok(())
}

pub struct CardSelectorPromptBuilder {
    scope: Scope,
    target: BrowserPromptTarget,
    subjects: Vec<CardId>,
    context: Option<PromptContext>,
    validation: Option<BrowserPromptValidation>,
    movement_effect: Option<Projectile>,
    visual_effect: Option<(GameObjectId, TimedEffectData)>,
    show_ability_alert: bool,
}

impl CardSelectorPromptBuilder {
    pub fn new(scope: Scope, target: BrowserPromptTarget) -> Self {
        Self {
            scope,
            target,
            subjects: vec![],
            context: None,
            validation: None,
            movement_effect: None,
            visual_effect: None,
            show_ability_alert: false,
        }
    }

    pub fn subjects(mut self, subjects: Vec<CardId>) -> Self {
        self.subjects = subjects;
        self
    }

    pub fn context(mut self, context: PromptContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn validation(mut self, validation: BrowserPromptValidation) -> Self {
        self.validation = Some(validation);
        self
    }

    pub fn movement_effect(mut self, movement_effect: Projectile) -> Self {
        self.movement_effect = Some(movement_effect);
        self
    }

    pub fn visual_effect(mut self, id: GameObjectId, effect: TimedEffectData) -> Self {
        self.visual_effect = Some((id, effect));
        self
    }

    pub fn show_ability_alert(mut self, show_ability_alert: bool) -> Self {
        self.show_ability_alert = show_ability_alert;
        self
    }
}
