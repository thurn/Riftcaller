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

use core_data::game_primitives::CardId;
use game_data::card_name::CardName;
use game_data::delegate_data::Scope;
use game_data::prompt_data::{
    FromZone, GamePrompt, PlayCardBrowser, PromptContext, UnplayedAction,
};

pub struct PlayCardBrowserBuilder {
    scope: Scope,
    cards: Vec<CardId>,
    from_zone: FromZone,
    context: PromptContext,
    unplayed_action: UnplayedAction,
}

impl PlayCardBrowserBuilder {
    pub fn new(scope: Scope, from_zone: FromZone, cards: Vec<CardId>) -> Self {
        Self {
            scope,
            cards,
            from_zone,
            context: PromptContext::PlayNamedCard(CardName::EchoingValor),
            unplayed_action: UnplayedAction::None,
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

    pub fn build(self) -> Option<GamePrompt> {
        Some(GamePrompt::PlayCardBrowser(PlayCardBrowser {
            context: Some(self.context),
            from_zone: self.from_zone,
            initiated_by: self.scope.ability_id(),
            cards: self.cards,
            unplayed_action: self.unplayed_action,
        }))
    }
}
