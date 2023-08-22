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

//! User interface actions

#![allow(clippy::use_self)] // Required to use EnumKind

use std::fmt;

use anyhow::{anyhow, Result};
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use crate::game::MulliganDecision;
use crate::primitives::{AbilityId, ActionCount, CardId, ManaValue, RoomId, Side};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SummonAction {
    /// Pay costs to summon the indicated minion during a raid, turning it
    /// face-up.
    SummonMinion(CardId),
    /// Do not pay the costs to summon a minion during a raid, and proceed to
    /// the next raid phase.
    DoNotSummmon,
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum EncounterAction {
    /// Defeat the minion being encountered with a weapon (source_id, target_id)
    UseWeaponAbility(CardId, CardId),
    /// Do not use a weapon and apply minion combat effects
    NoWeapon,
    /// Custom card action, resolved and then treated equivalently to 'no
    /// weapon'
    CardAction(CardPromptAction),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum RazeCardActionType {
    /// Raze a card in play
    Destroy,
    /// Raze a card in the sanctum or vault
    Discard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum AccessPhaseAction {
    ScoreCard(CardId),
    RazeCard(CardId, RazeCardActionType, ManaValue),
    EndRaid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromptContext {
    /// Prompt is being shown related to a specific card
    Card(CardId),
}

/// A choice which can be made as part of an ability of an individual card
///
/// Maybe switch this to a trait someday?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum CardPromptAction {
    /// A player loses mana
    LoseMana(Side, ManaValue),
    /// A player loses action points
    LoseActions(Side, ActionCount),
    /// End the current raid in failure.
    EndRaid,
    /// Deal damage to the Champion
    TakeDamage(AbilityId, u32),
    /// Deal damage and end the current raid
    TakeDamageEndRaid(AbilityId, u32),
}

/// An action which can be taken in the user interface, typically embedded
/// inside the `GameAction::StandardAction` protobuf message type when sent to
/// the client.
#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum PromptAction {
    /// Action to keep or mulligan opening hand
    MulliganDecision(MulliganDecision),
    /// Action for a player to end their turn.
    EndTurnAction,
    /// Action for a player to begin their next turn.
    StartTurnAction,
    /// Overlord action during a raid to decide whether to summon a defending
    /// minion.
    SummonAction(SummonAction),
    /// Champion action in response to a raid encounter
    EncounterAction(EncounterAction),
    /// Action to target & destroy an accessed card
    AccessPhaseAction(AccessPhaseAction),
    /// Action to take as part of a card ability
    CardAction(CardPromptAction),
}

impl fmt::Debug for PromptAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MulliganDecision(d) => write!(f, "{d:?}"),
            Self::StartTurnAction => write!(f, "StartTurn"),
            Self::EndTurnAction => write!(f, "EndTurn"),
            Self::SummonAction(a) => write!(f, "{a:?}"),
            Self::EncounterAction(a) => write!(f, "{a:?}"),
            Self::AccessPhaseAction(a) => write!(f, "{a:?}"),
            Self::CardAction(a) => write!(f, "{a:?}"),
        }
    }
}

/// Presents a choice to a user, typically communicated via a series of buttons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePrompt {
    /// Identifies the context for this prompt, i.e. why it is being shown to
    /// the user
    pub context: Option<PromptContext>,
    /// Possible responses to this prompt
    pub responses: Vec<PromptAction>,
}

impl GamePrompt {
    pub fn card_actions(actions: Vec<CardPromptAction>) -> Self {
        Self {
            context: None,
            responses: actions.into_iter().map(PromptAction::CardAction).collect(),
        }
    }
}

/// Possible targets for the 'play card' action. Note that many types of targets
/// are *not* selected in the original PlayCard action request but are instead
/// selected via a follow-up prompt, and thus are not represented here.
#[derive(
    PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, EnumKind, Ord, PartialOrd,
)]
#[enum_kind(CardTargetKind)]
pub enum CardTarget {
    None,
    Room(RoomId),
}

impl CardTarget {
    /// Gets the RoomId targeted by a player, or returns an error if no target
    /// was provided.
    pub fn room_id(&self) -> Result<RoomId> {
        match self {
            CardTarget::Room(room_id) => Ok(*room_id),
            _ => Err(anyhow!("Expected a RoomId to be provided but got {:?}", self)),
        }
    }
}

/// Possible actions a player can take to mutate a GameState
#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum GameAction {
    PromptAction(PromptAction),
    Resign,
    GainMana,
    DrawCard,
    PlayCard(CardId, CardTarget),
    ActivateAbility(AbilityId, CardTarget),
    UnveilCard(CardId),
    InitiateRaid(RoomId),
    LevelUpRoom(RoomId),
    SpendActionPoint,
}

impl fmt::Debug for GameAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PromptAction(prompt) => write!(f, "@{prompt:?}"),
            Self::Resign => write!(f, "@Resign"),
            Self::GainMana => write!(f, "@GainMana"),
            Self::DrawCard => write!(f, "@DrawCard"),
            Self::PlayCard(id, target) => {
                f.debug_tuple("@PlayCard").field(id).field(target).finish()
            }
            Self::ActivateAbility(id, target) => {
                f.debug_tuple("@ActivateAbility").field(id).field(target).finish()
            }
            Self::UnveilCard(id) => f.debug_tuple("@UnveilCard").field(id).finish(),
            Self::InitiateRaid(arg0) => f.debug_tuple("@InitiateRaid").field(arg0).finish(),
            Self::LevelUpRoom(arg0) => f.debug_tuple("@LevelUpRoom").field(arg0).finish(),
            Self::SpendActionPoint => write!(f, "@SpendActionPoint"),
        }
    }
}
