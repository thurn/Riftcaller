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

use core_data::game_primitives::AbilityId;
use enum_kinds::EnumKind;
use game_data::card_configuration::{Cost, TargetRequirement};
use game_data::delegate_data::GameDelegate;
use game_data::text::TextElement;

/// Possible types of ability
#[derive(Debug, Clone, EnumKind)]
#[enum_kind(AbilityTypeKind)]
pub enum AbilityType {
    /// Standard abilities function at all times without requiring activation.
    Standard,

    /// Activated abilities have an associated cost in order to be used.
    Activated { cost: Cost<AbilityId>, target_requirement: TargetRequirement<AbilityId> },
}

/// Abilities are the unit of action in Riftcaller. Their behavior is provided
/// by the Delegate system, see delegate_data for more information.
#[derive(Debug)]
pub struct Ability {
    pub ability_type: AbilityType,
    pub text: Vec<TextElement>,
    pub delegates: Vec<GameDelegate>,
}

impl Ability {
    pub fn new(text: Vec<TextElement>) -> Self {
        Self { ability_type: AbilityType::Standard, text, delegates: vec![] }
    }

    pub fn new_with_delegate(text: Vec<TextElement>, delegate: GameDelegate) -> Self {
        Self { ability_type: AbilityType::Standard, text, delegates: vec![delegate] }
    }

    pub fn delegate(mut self, delegate: GameDelegate) -> Self {
        self.delegates.push(delegate);
        self
    }
}

/// Builder helper for activated abilities
#[derive(Debug)]
pub struct ActivatedAbility {
    cost: Cost<AbilityId>,
    text: Vec<TextElement>,
    target_requirement: TargetRequirement<AbilityId>,
    delegates: Vec<GameDelegate>,
}

impl ActivatedAbility {
    pub fn new(cost: Cost<AbilityId>, text: Vec<TextElement>) -> Self {
        Self { cost, text, target_requirement: TargetRequirement::None, delegates: vec![] }
    }

    pub fn target_requirement(mut self, requirement: TargetRequirement<AbilityId>) -> Self {
        self.target_requirement = requirement;
        self
    }

    pub fn delegate(mut self, delegate: GameDelegate) -> Self {
        self.delegates.push(delegate);
        self
    }

    pub fn build(self) -> Ability {
        Ability {
            ability_type: AbilityType::Activated {
                cost: self.cost,
                target_requirement: self.target_requirement,
            },
            text: self.text,
            delegates: self.delegates,
        }
    }
}
