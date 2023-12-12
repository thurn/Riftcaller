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

use core_data::game_primitives::{CardId, InitiatedBy};
use game_data::delegate_data::Scope;
use game_data::prompt_data::{
    BrowserPromptTarget, BrowserPromptValidation, CardSelectorPrompt, GamePrompt, PromptContext,
};

pub struct CardSelectorPromptBuilder {
    initiated_by: InitiatedBy,
    target: BrowserPromptTarget,
    subjects: Vec<CardId>,
    context: Option<PromptContext>,
    validation: Option<BrowserPromptValidation>,
    can_reorder: bool,
}

impl CardSelectorPromptBuilder {
    pub fn new(scope: Scope, target: BrowserPromptTarget) -> Self {
        Self {
            initiated_by: scope.initiated_by(),
            target,
            subjects: vec![],
            context: None,
            validation: None,
            can_reorder: false,
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

    pub fn can_reorder(mut self, can_reorder: bool) -> Self {
        self.can_reorder = can_reorder;
        self
    }

    /// Display a new Card Selector prompt a [CardSelectorPromptBuilder].
    ///
    /// Has no effect if no subject cards have been specified for this selector.
    pub fn build(self) -> Option<GamePrompt> {
        if self.subjects.is_empty() {
            return None;
        }

        Some(GamePrompt::CardSelector(CardSelectorPrompt {
            initiated_by: self.initiated_by,
            context: self.context,
            unchosen_subjects: self.subjects,
            chosen_subjects: vec![],
            target: self.target,
            validation: self.validation,
            can_reorder: self.can_reorder,
        }))
    }
}
