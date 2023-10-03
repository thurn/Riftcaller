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

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::delegate_data::RaidEvent;
use crate::game_actions::RazeCardActionType;
use crate::game_state::RaidJumpRequest;
use crate::game_updates::InitiatedBy;
use crate::primitives::{CardId, ManaValue, RaidId, RoomId, Side};

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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RaidStep {
    Begin,
    NextEncounter,
    PopulateSummonPrompt(CardId),
    SummonMinion(CardId),
    DoNotSummon(CardId),
    EncounterMinion(CardId),
    PopulateEncounterPrompt(CardId),
    UseWeapon(WeaponInteraction),
    MinionDefeated(WeaponInteraction),
    FireMinionCombatAbility(CardId),
    PopulateApproachPrompt,
    AccessStart,
    BuildAccessSet,
    AccessSetBuilt,
    RevealAccessedCards,
    AccessCards,
    PopulateAccessPrompt,
    StartScoringCard(ScoredCard),
    ChampionScoreEvent(ScoredCard),
    ScoreEvent(ScoredCard),
    ScorePointsForCard(ScoredCard),
    MoveToScoredPosition(ScoredCard),
    StartRazingCard(CardId, ManaValue),
    RazeCard(CardId, ManaValue),
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
            RaidStatus::Begin => Side::Champion,
            RaidStatus::Summon => Side::Overlord,
            RaidStatus::Encounter => Side::Champion,
            RaidStatus::ApproachRoom => Side::Overlord,
            RaidStatus::Access => Side::Champion,
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

    pub fn prompt(prompt_type: RaidStatus, choices: Vec<RaidChoice>) -> Result<RaidState> {
        Ok(Self::Prompt(RaidPrompt { status: prompt_type, choices }))
    }
}

/// A subset of raid information that can be conveniently passed around.
#[derive(Debug, Clone, Copy)]
pub struct RaidInfo {
    pub raid_id: RaidId,
    pub target: RoomId,
    pub encounter: usize,
}

impl RaidInfo {
    pub fn event(&self) -> RaidEvent {
        RaidEvent { raid_id: self.raid_id, target: self.target }
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
    /// Current encounter position within this raid
    pub encounter: usize,
    /// Cards which have been accessed as part of this raid's Access phase.
    pub accessed: Vec<CardId>,
    /// Requested new state for this raid. See [RaidJumpRequest] for details.
    pub jump_request: Option<RaidJumpRequest>,
}

impl RaidData {
    pub fn info(&self) -> RaidInfo {
        RaidInfo { raid_id: self.raid_id, target: self.target, encounter: self.encounter }
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