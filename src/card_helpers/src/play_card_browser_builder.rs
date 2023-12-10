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

use core_data::game_primitives::{CardId, GameObjectId};
use game_data::delegate_data::Scope;
use game_data::prompt_data::{GamePrompt, PlayCardBrowser, PromptContext, UnplayedAction};
use game_data::special_effects::{Projectile, TimedEffectData};

pub fn show(builder: PlayCardBrowserBuilder) -> Option<GamePrompt> {
    // let mut effects = VisualEffects::new();
    // if let Some((id, data)) = builder.visual_effect {
    //     effects = effects.timed_effect(id, data);
    // }

    // if let Some(movement_effects) = builder.movement_effect {
    //     effects.card_movement_effects(movement_effects, &cards).apply(game);
    // }
    //
    // game.add_animation(|| GameAnimation::ShowPlayCardBrowser(cards));

    Some(GamePrompt::PlayCardBrowser(PlayCardBrowser {
        context: Some(builder.context),
        initiated_by: builder.scope.ability_id(),
        cards: builder.cards,
        unplayed_action: builder.unplayed_action,
    }))
}

pub struct PlayCardBrowserBuilder {
    scope: Scope,
    cards: Vec<CardId>,
    context: PromptContext,
    unplayed_action: UnplayedAction,
    movement_effect: Option<Projectile>,
    visual_effect: Option<(GameObjectId, TimedEffectData)>,
}

impl PlayCardBrowserBuilder {
    pub fn new(scope: Scope, cards: Vec<CardId>) -> Self {
        Self {
            scope,
            cards,
            context: PromptContext::PlayACard,
            unplayed_action: UnplayedAction::None,
            movement_effect: None,
            visual_effect: None,
        }
    }

    pub fn context(mut self, context: PromptContext) -> Self {
        self.context = context;
        self
    }

    pub fn unplayed_action(mut self, unplayed_action: UnplayedAction) -> Self {
        self.unplayed_action = unplayed_action;
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
}
