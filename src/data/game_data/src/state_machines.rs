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

use core_data::game_primitives::{AbilityId, CardId, DamageAmount, InitiatedBy, Side};
use serde::{Deserialize, Serialize};

use crate::game_actions::CardTarget;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PlayCardStep {
    Begin,
    CheckLimits,
    ClearPreviousState,
    AddToHistory,
    MoveToPlayedPosition,
    PayActionPoints,
    ApplyPlayCardBrowser,
    PayManaCost,
    PayCustomCost,
    TurnFaceUp,
    MoveToTargetPosition,
    Finish,
}

/// Data related to an ongoing action to play a card.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PlayCardOptions {
    pub ignore_action_cost: bool,
    pub ignore_mana_cost: bool,
}

/// Data related to an ongoing action to play a card.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayCardData {
    /// Card being played
    pub card_id: CardId,
    /// How this card play was started
    pub initiated_by: InitiatedBy,
    /// Room being targeted, if any
    pub target: CardTarget,
    /// Configuration options for playing this card.
    pub options: PlayCardOptions,
    /// Current state machine state
    pub step: PlayCardStep,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActivateAbilityStep {
    Begin,
    PayActionPoints,
    PayManaCost,
    PayCustomCost,
    Finish,
}

/// Data related to an ongoing action to play a card.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ActivateAbilityData {
    /// Ability being activated
    pub ability_id: AbilityId,
    /// Room being targeted, if any
    pub target: CardTarget,
    /// Current state machine state
    pub step: ActivateAbilityStep,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DealDamageStep {
    Begin,
    WillDealDamageEvent,
    DiscardCards,
    DealtDamageEvent,
    Finish,
}

/// State data for dealing damage to the Champion player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealDamageData {
    /// Amount of damage to deal
    pub amount: DamageAmount,
    /// Source of the damage
    pub source: AbilityId,
    /// Cards which have been discarded to this damage event, if any
    pub discarded: Vec<CardId>,
    /// Current state machine state
    pub step: DealDamageStep,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GiveCursesStep {
    Begin,
    WillReceiveCursesEvent,
    AddCurses,
    CursesReceivedEvent,
    Finish,
}

/// State data for giving a curse to the Champion player
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GiveCursesData {
    /// Number of curses to add
    pub quantity: u32,
    /// Source of the curses
    pub source: AbilityId,
    /// Current state machine state
    pub step: GiveCursesStep,
}

type DrawCardsCount = u32;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DrawCardsStep {
    Begin,
    WillDrawCardsEvent,
    CheckIfDrawPrevented,
    DrawCards,
    DrawCardsViaAbilityEvent(DrawCardsCount),
    AddToHistory(DrawCardsCount),
    Finish,
}

/// State data for drawing cards
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DrawCardsData {
    /// Which player is drawing cards?
    pub side: Side,
    /// Number of cards to draw
    pub quantity: u32,
    /// If true, the draw event has been prevented and no cards will be drawn.
    pub draw_is_prevented: bool,
    /// Source of the card draw
    pub source: InitiatedBy,
    /// Current state machine state
    pub step: DrawCardsStep,
}

/// Data related to ongoing game events. Some types of updates are handled via a
/// resumable state machine in order to allow interruptions in the resolution
/// process when a player is required to make a prompt decision.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StateMachines {
    pub play_card: Option<PlayCardData>,
    pub activate_ability: Option<ActivateAbilityData>,
    pub deal_damage: Option<DealDamageData>,
    pub give_curses: Vec<GiveCursesData>,
    pub draw_cards: Vec<DrawCardsData>,
}
