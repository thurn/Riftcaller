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

use std::collections::HashMap;

use core_data::adventure_primitives::{NarrativeChoiceId, NarrativeEventId, Skill};
use core_data::game_primitives::Sprite;
use game_data::card_name::CardVariant;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::adventure_action::NarrativeEffectIndex;
use crate::adventure_effect_data::AdventureEffectData;

/// One possible choice within a narrative event screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEventChoice {
    /// Narrative description of the action for this choice.
    pub choice_description: String,
    /// Narrative description of the outcome of this choice.
    pub result_description: String,
    /// Skill required to select this choice, if any.
    pub skill: Option<Skill>,
    /// Costs to select this choice.
    ///
    /// Choices will not be presented unless the player is able to pay all of
    /// their associated costs.
    pub costs: Vec<AdventureEffectData>,
    /// Rewards for selecting this choice.
    pub rewards: Vec<AdventureEffectData>,
}

impl NarrativeEventChoice {
    pub fn effect(&self, index: NarrativeEffectIndex) -> &AdventureEffectData {
        match index {
            NarrativeEffectIndex::Cost(i) => &self.costs[i],
            NarrativeEffectIndex::Reward(i) => &self.rewards[i],
        }
    }

    pub fn enumerate_costs(
        &self,
    ) -> impl Iterator<Item = (NarrativeEffectIndex, &AdventureEffectData)> {
        self.costs.iter().enumerate().map(|(i, choice)| (NarrativeEffectIndex::Cost(i), choice))
    }

    pub fn enumerate_rewards(
        &self,
    ) -> impl Iterator<Item = (NarrativeEffectIndex, &AdventureEffectData)> {
        self.rewards.iter().enumerate().map(|(i, choice)| (NarrativeEffectIndex::Reward(i), choice))
    }
}

/// Steps within the progress of resolving a narrative event.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum NarrativeEventStep {
    /// Introductory text for this event.
    Introduction,
    /// View valid narrative choices for this event which have not yet been
    /// selected.
    ViewChoices,
    /// View the result of selecting the narrative choice with the provided
    /// [NarrativeChoiceId].
    SelectChoice(NarrativeChoiceId),
}

/// Static Data for displaying a narrative event to the player with a fixed set
/// of choices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEventData {
    /// Image associated with this narrative event
    pub image: Sprite,
    /// Narrative description introducing this event.
    pub description: String,
    /// List of possible choices within this narrative event, indexed by
    /// [NarrativeChoiceId].
    pub choices: HashMap<NarrativeChoiceId, NarrativeEventChoice>,
}

impl NarrativeEventData {
    pub fn enumerate_choices(
        &self,
    ) -> impl Iterator<Item = (NarrativeChoiceId, &NarrativeEventChoice)> {
        self.choices.iter().map(|(id, choice)| (*id, choice))
    }

    pub fn choice(&self, id: NarrativeChoiceId) -> &NarrativeEventChoice {
        self.choices.get(&id).unwrap_or_else(|| panic!("Narrative choice not found {id:?}"))
    }

    pub fn choice_mut(&mut self, id: NarrativeChoiceId) -> &mut NarrativeEventChoice {
        self.choices.get_mut(&id).unwrap_or_else(|| panic!("Narrative choice not found {id:?}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NarrativeEffectState {
    /// Has this effect been applied by the user?
    pub applied: bool,
    /// Optionally, a card name associated with this effect which is known in
    /// advance of making the decision to select this option.
    pub known_card: Option<CardVariant>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NarrativeChoiceState {
    /// Has the user selected to take this choice?
    pub selected: bool,
    /// State of effects within this choice
    #[serde_as(as = "Vec<(_, _)>")]
    pub effects: HashMap<NarrativeEffectIndex, NarrativeEffectState>,
}

impl NarrativeChoiceState {
    pub fn effect(&self, id: NarrativeEffectIndex) -> &NarrativeEffectState {
        static DEFAULT: NarrativeEffectState =
            NarrativeEffectState { applied: false, known_card: None };
        self.effects.get(&id).unwrap_or(&DEFAULT)
    }

    pub fn effect_mut(&mut self, id: NarrativeEffectIndex) -> &mut NarrativeEffectState {
        self.effects.entry(id).or_default()
    }
}

/// State for an ongoing narrative event screen
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEventState {
    /// Identifier for the current narrative event
    pub id: NarrativeEventId,
    /// Current screen within the event.
    pub step: NarrativeEventStep,
    /// State of choices within this narrative event
    #[serde_as(as = "Vec<(_, _)>")]
    pub choices: HashMap<NarrativeChoiceId, NarrativeChoiceState>,
}

static DEFAULT_CHOICE_STATE: Lazy<NarrativeChoiceState> =
    Lazy::new(|| NarrativeChoiceState::default());

impl NarrativeEventState {
    pub fn choice(&self, id: NarrativeChoiceId) -> &NarrativeChoiceState {
        self.choices.get(&id).unwrap_or(&DEFAULT_CHOICE_STATE)
    }

    pub fn choice_mut(&mut self, id: NarrativeChoiceId) -> &mut NarrativeChoiceState {
        self.choices.entry(id).or_default()
    }
}
