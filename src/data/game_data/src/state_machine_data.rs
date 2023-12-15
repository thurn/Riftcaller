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

use core_data::game_primitives::{
    AbilityId, CardId, CardPlayId, DamageAmount, InitiatedBy, Side, WoundCount,
};
use serde::{Deserialize, Serialize};

use crate::game_actions::CardTarget;
use crate::game_state::TurnData;
use crate::prompt_data::FromZone;

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
    /// Origin zone for the card being played
    pub from_zone: FromZone,
    /// How this card play was started
    pub initiated_by: InitiatedBy,
    /// Room being targeted, if any
    pub target: CardTarget,
    /// Unique identifier for this instance of this card being played.
    pub card_play_id: CardPlayId,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DealDamageStep {
    Begin,
    WillDealDamageEvent,
    DiscardCards,
    DealtDamageEvent(Vec<CardId>),
    Finish,
}

/// Data about an ongoing damage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealDamageData {
    /// Amount of damage to deal
    pub amount: DamageAmount,
    /// Source of the damage
    pub source: AbilityId,
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

/// Options for giving curses to the Riftcaller
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct GiveCurseOptions {
    pub for_turn: Option<TurnData>,
}

/// State data for giving a curse to the Riftcaller player
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GiveCursesData {
    /// Number of curses to add
    pub quantity: u32,
    /// Source of the curses
    pub source: AbilityId,
    /// Configuration options
    pub options: GiveCurseOptions,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GiveLeylinesStep {
    Begin,
    AddLeylines,
    LeylinesReceivedEvent,
    Finish,
}

/// State data for giving a leyline to the Riftcaller player
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GiveLeylinesData {
    /// Number of leylines to add
    pub quantity: u32,
    /// Source of the leylines
    pub source: AbilityId,
    /// Current state machine state
    pub step: GiveLeylinesStep,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GiveWoundsStep {
    Begin,
    AddWounds,
    WoundsReceivedEvent,
    Finish,
}

/// State data for giving a wound to the Riftcaller player
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GiveWoundsData {
    /// Number of wounds to give
    pub quantity: WoundCount,
    /// Source of the wounds
    pub source: AbilityId,
    /// Current state machine state
    pub step: GiveWoundsStep,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DestroyPermanentStep {
    Begin,
    WillDestroyEvent,
    CheckIfDestroyPrevented,
    Destroy,
    CardsDestroyedEvent,
    Finish,
}

/// State data for destroying a card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestroyPermanentsData {
    /// Target permanent(s) to destroy.
    pub targets: Vec<CardId>,
    /// Source of the event
    pub source: InitiatedBy,
    /// Current state machine state
    pub step: DestroyPermanentStep,
}

/// Data related to ongoing game events. Some types of updates are handled via a
/// resumable state machine in order to allow interruptions in the resolution
/// process when a player is required to make a prompt decision.
///
/// See the `state_machine` module for more information.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StateMachines {
    pub play_card: Vec<PlayCardData>,
    pub activate_ability: Vec<ActivateAbilityData>,
    pub deal_damage: Vec<DealDamageData>,
    pub give_curses: Vec<GiveCursesData>,
    pub draw_cards: Vec<DrawCardsData>,
    pub give_leylines: Vec<GiveLeylinesData>,
    pub give_wounds: Vec<GiveWoundsData>,
    pub destroy_permanent: Vec<DestroyPermanentsData>,
}
