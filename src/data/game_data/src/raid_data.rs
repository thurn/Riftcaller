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
use core_data::game_primitives::{
    CardId, InitiatedBy, ManaValue, MinionEncounterId, RaidId, RoomAccessId, RoomId, Side,
};
use serde::{Deserialize, Serialize};

use crate::delegate_data::{AccessEvent, RaidEvent};
use crate::game_actions::RazeCardActionType;
use crate::game_state::RaidJumpRequest;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct WeaponInteraction {
    pub weapon_id: CardId,
    pub defender_id: CardId,
}

impl WeaponInteraction {
    pub fn new(source: CardId, target: CardId) -> Self {
        Self { weapon_id: source, defender_id: target }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScoredCard {
    pub id: CardId,
}

impl ScoredCard {
    pub fn new(id: CardId) -> Self {
        Self { id }
    }
}

/// Identifies the reason why we have entered the 'populate access prompt' step.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PopulateAccessPromptSource {
    Initial,
    FromScore,
    FromRaze,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RaidStep {
    Begin,
    GainLeylineMana,
    RaidStartEvent,
    NextEncounter,
    PopulateSummonPrompt(CardId),
    SummonMinion(CardId),
    DoNotSummon(CardId),
    ApproachMinion(CardId),
    EncounterMinion(CardId),
    PopulateEncounterPrompt(CardId),
    UseWeapon(WeaponInteraction),
    MinionDefeated(WeaponInteraction),
    FireMinionCombatAbility(CardId),
    PopulateApproachPrompt,
    AccessStart,
    CheckIfCardAccessPrevented,
    BuildAccessSet,
    AccessSetBuilt,
    RevealAccessedCards,
    AccessCards,
    WillPopulateAccessPrompt(PopulateAccessPromptSource),
    PopulateAccessPrompt,
    StartScoringCard(ScoredCard),
    RiftcallerScoreEvent(ScoredCard),
    ScoreEvent(ScoredCard),
    MoveToScoredPosition(ScoredCard),
    StartRazingCard(CardId, ManaValue),
    RazeCard(CardId, ManaValue),
    FinishAccess,
    FinishRaid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum RaidLabel {
    SummonMinion(CardId),
    DoNotSummonMinion,
    UseWeapon(WeaponInteraction),
    DoNotUseWeapon,
    ProceedToAccess,
    ScoreCard(CardId),
    RazeCard(CardId, RazeCardActionType),
    EndRaid,
    EndAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaidChoice {
    pub label: RaidLabel,
    pub step: RaidStep,
}

impl RaidChoice {
    pub fn new(label: RaidLabel, step: RaidStep) -> Self {
        Self { label, step }
    }
}

/// Describes what is happening at a high level in a raid, e.g. which player can
/// currently act.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RaidStatus {
    Begin,
    Summon,
    Encounter,
    ApproachRoom,
    Access,
}

impl RaidStatus {
    pub fn side(&self) -> Side {
        match self {
            RaidStatus::Begin => Side::Riftcaller,
            RaidStatus::Summon => Side::Covenant,
            RaidStatus::Encounter => Side::Riftcaller,
            RaidStatus::ApproachRoom => Side::Covenant,
            RaidStatus::Access => Side::Riftcaller,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaidPrompt {
    /// Identifies which prompt this is and how the raid state should be
    /// displayed in the UI.
    pub status: RaidStatus,
    /// A list of choices the player can select between. Whichever option is
    /// picked becomes the new [RaidStep] for this raid and the state machine
    /// continues.
    pub choices: Vec<RaidChoice>,
    /// Name of the [RaidStep] which populates the prompt choices here.
    ///
    /// Because user actions may modify the set of available raid actions, we
    /// rerun the raid state machine logic to rebuild the prompt after each user
    /// action while viewing a raid prompt.
    pub populated_by: RaidStep,
}

/// State of an ongoing raid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RaidState {
    Step(RaidStep),
    Prompt(RaidPrompt),
}

impl RaidState {
    pub fn step(step: RaidStep) -> Result<RaidState> {
        Ok(Self::Step(step))
    }

    pub fn prompt(
        prompt_type: RaidStatus,
        populated_by: RaidStep,
        choices: Vec<RaidChoice>,
    ) -> Result<RaidState> {
        Ok(Self::Prompt(RaidPrompt { status: prompt_type, populated_by, choices }))
    }
}

/// A subset of raid information that can be conveniently passed around.
#[derive(Debug, Clone, Copy)]
pub struct RaidInfo {
    pub raid_id: RaidId,
    pub initiated_by: InitiatedBy,
    pub target: RoomId,
    pub encounter: usize,
    pub minion_encounter_id: Option<MinionEncounterId>,
    pub room_access_id: Option<RoomAccessId>,
    pub is_card_access_prevented: bool,
    pub is_custom_access: bool,
}

impl RaidInfo {
    pub fn event<T>(&self, data: T) -> RaidEvent<T> {
        RaidEvent {
            raid_id: self.raid_id,
            target: self.target,
            minion_encounter_id: self.minion_encounter_id,
            room_access_id: self.room_access_id,
            data,
        }
    }

    pub fn access_event<T>(&self, data: T) -> AccessEvent<T> {
        if self.is_custom_access {
            AccessEvent::CustomCardAccess(data)
        } else {
            AccessEvent::RaidAccess(self.event(data))
        }
    }
}

/// Data about an active raid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaidData {
    /// Unique ID for this raid
    pub raid_id: RaidId,
    /// Source which initiated this raid
    pub initiated_by: InitiatedBy,
    /// Room being targeted by this raid
    pub target: RoomId,
    /// Current state of this raid. Use the functions in the `raid_state` crate
    /// instead of directly inspecting this value.
    pub state: RaidState,
    /// Current encounter position within this raid. Index 0 is the innermost
    /// defender.
    pub encounter: usize,
    /// Identifier for the current minion encounter, if any
    pub minion_encounter_id: Option<MinionEncounterId>,
    /// Identifier for the current room access, if any
    pub room_access_id: Option<RoomAccessId>,
    /// Cards which have been accessed as part of this raid's Access phase.
    pub accessed: Vec<CardId>,
    /// Requested new state for this raid. See [RaidJumpRequest] for details.
    pub jump_request: Option<RaidJumpRequest>,
    /// True if card access is prevented for this raid.
    ///
    /// This is usually described with the phrase "instead of accessing cards,
    /// do X". This will still fire `RaidAccessStartEvent` and
    /// `RaidAccessEndEvent`, but will not trigger e.g.
    /// `RaidAccessSelectedEvent`.
    ///
    /// It is fine to stack multiple replacement effects like this for one raid.
    /// Other raid steps still happen, e.g. the raid is still considered
    /// "successful", you have still "accessed the room this turn", and you
    /// can retarget the raid to different rooms.
    pub is_card_access_prevented: bool,
    /// A custom access raid plays out only the 'access' phase of a raid,
    /// accessing a specific set of cards.
    pub is_custom_access: bool,
}

impl RaidData {
    pub fn info(&self) -> RaidInfo {
        RaidInfo {
            raid_id: self.raid_id,
            initiated_by: self.initiated_by,
            target: self.target,
            encounter: self.encounter,
            minion_encounter_id: self.minion_encounter_id,
            room_access_id: self.room_access_id,
            is_card_access_prevented: self.is_card_access_prevented,
            is_custom_access: self.is_custom_access,
        }
    }
}

/// Represents how the current state of a raid should be represented in the user
/// interface -- with no content, as a sequence of defenders, or by showing
/// accessed cards.
pub enum RaidDisplayState {
    None,
    Defenders(Vec<CardId>),
    Access,
}
