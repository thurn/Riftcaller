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

//! User interface actions

#![allow(clippy::use_self)] // Required to use EnumKind

use std::fmt;

use anyhow::{anyhow, Result};
use core_data::game_primitives::{
    AbilityId, CardId, CardSubtype, CardType, CurseCount, DamageAmount, ManaValue, RoomId,
};
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use crate::game_state::MulliganDecision;
use crate::prompt_data::{PromptAction, PromptContext};

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
    /// Invoke an additional custom action associated with this minion at the
    /// provided index in its additional actions list.
    AdditionalAction(usize),
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ApproachRoomAction {
    /// Continue to the room acces phase.
    Proceed,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ButtonPromptContext {
    /// Prompt is being shown related to a specific card
    Card(CardId),
    /// Prompt is being shown to sacrifice cards due to exceeding the
    /// limit for cards in play of this type. Player must sacrifice until they
    /// have the provided number of minions in the room.
    CardLimit(CardType, Option<CardSubtype>),
    /// A player has a priority window in which to activate abilities
    PriorityWindow,
    /// Sacrifice a card to prevent up to `DamageAmount` damage. Will inspect
    /// the current incoming damage value and display only the lower of the two
    /// values.
    SacrificeToPreventDamage(CardId, DamageAmount),
    /// Sacrifice a card to prevent up to `CurseCount` curses. Will inspect
    /// the current incoming curse count and display only the lower of the two
    /// values.
    SacrificeToPreventCurses(CardId, CurseCount),
    /// Prompt to sacrifice the [CardId] card to prevent the a card from
    /// being destroyed.
    SacrificeToPreventDestroyingCard(CardId),
    /// Prompt for a card to give to the opponent
    CardToGiveToOpponent,
    /// Prompt for a card to take from the opponent
    CardToTakeFromOpponent,
    /// Prompt to pay mana to prevent revealing a card
    PayToPreventRevealing(ManaValue),
    /// Prompt from the indicated card to pick a card type
    ChooseCardType(CardId),
}

impl ButtonPromptContext {
    /// Looks up the card associated with this prompt, if any
    pub fn associated_card(&self) -> Option<CardId> {
        match self {
            Self::Card(id) => Some(*id),
            Self::SacrificeToPreventDamage(id, _) => Some(*id),
            Self::SacrificeToPreventCurses(id, _) => Some(*id),
            Self::ChooseCardType(id) => Some(*id),
            _ => None,
        }
    }
}

/// An action which can be taken in the user interface as a result of the game
/// rules (current game state) and not because of any particular cards in play.
#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum GameStateAction {
    /// Action to keep or mulligan opening hand
    MulliganDecision(MulliganDecision),
    /// Action for a player to end their turn.
    EndTurnAction,
    /// Action for a player to begin their next turn.
    StartTurnAction,
}

impl fmt::Debug for GameStateAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MulliganDecision(d) => write!(f, "{d:?}"),
            Self::StartTurnAction => write!(f, "StartTurn"),
            Self::EndTurnAction => write!(f, "EndTurn"),
        }
    }
}

/// Presents a choice to a user, typically communicated via a series of buttons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionButtons {
    /// Identifies the context for this prompt, i.e. why it is being shown to
    /// the user
    pub context: Option<PromptContext>,
    /// Possible responses to this prompt
    pub responses: Vec<GameStateAction>,
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

impl From<RoomId> for CardTarget {
    fn from(value: RoomId) -> Self {
        CardTarget::Room(value)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RaidAction {
    /// Index position of the action to take in the current `RaidStep`.
    pub index: usize,
}

/// Configuration options for how the game is rendered which do not affect game
/// logic.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum DisplayPreference {
    ShowArenaView(bool),
}

/// Possible actions a player can take to mutate a GameState
#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum GameAction {
    GameStateAction(GameStateAction),
    Resign,
    GainMana,
    DrawCard,
    PlayCard(CardId, CardTarget),
    ActivateAbility(AbilityId, CardTarget),
    SummonProject(CardId),
    RemoveCurse,
    DispelEvocation,
    InitiateRaid(RoomId),
    ProgressRoom(RoomId),
    SpendActionPoint,
    MoveSelectorCard { card_id: CardId, index: Option<u32> },
    RaidAction(RaidAction),
    PromptAction(PromptAction),
    SetDisplayPreference(DisplayPreference),
}

impl GameAction {
    /// Returns true if this action should not cause state-based actions to run.
    pub fn is_stateless_action(&self) -> bool {
        match self {
            Self::SetDisplayPreference(..) => true,
            _ => false,
        }
    }
}

impl fmt::Debug for GameAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameStateAction(action) => write!(f, "@{action:?}"),
            Self::Resign => write!(f, "@Resign"),
            Self::GainMana => write!(f, "@GainMana"),
            Self::DrawCard => write!(f, "@DrawCard"),
            Self::PlayCard(id, target) => {
                f.debug_tuple("@PlayCard").field(id).field(target).finish()
            }
            Self::ActivateAbility(id, target) => {
                f.debug_tuple("@ActivateAbility").field(id).field(target).finish()
            }
            Self::SummonProject(id) => f.debug_tuple("@SummonProject").field(id).finish(),
            Self::RemoveCurse => write!(f, "@RemoveCurse"),
            Self::DispelEvocation => write!(f, "@DispelEvocation"),
            Self::InitiateRaid(arg0) => f.debug_tuple("@InitiateRaid").field(arg0).finish(),
            Self::ProgressRoom(arg0) => f.debug_tuple("@ProgressRoom").field(arg0).finish(),
            Self::SpendActionPoint => write!(f, "@SpendActionPoint"),
            Self::MoveSelectorCard { card_id, index } => {
                f.debug_tuple("@MoveCard").field(card_id).field(index).finish()
            }
            Self::RaidAction(action) => f.debug_tuple("@RaidAction").field(&action.index).finish(),
            Self::PromptAction(prompt) => write!(f, "@{prompt:?}"),
            Self::SetDisplayPreference(preference) => {
                f.debug_tuple("@SetDisplayPreference").field(preference).finish()
            }
        }
    }
}
