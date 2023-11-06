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

use crate::game_effect::GameEffect;
use crate::game_state::MulliganDecision;
use crate::primitives::{
    AbilityId, CardId, CardSubtype, CardType, CurseCount, DamageAmount, ManaValue, RoomId,
};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromptContext {
    /// Prompt is being shown related to a specific card
    Card(CardId),
    /// Prompt is being show to discard cards due to exceeding the hand size
    /// limit, player must discard until they have the provided number of cards
    /// in hand.
    DiscardToHandSize(usize),
    /// Play a chosen card
    PlayACard,
    /// Play a card of a given type the discard pile
    PlayFromDiscard(CardType),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ButtonPromptContext {
    /// Prompt is being shown related to a specific card
    Card(CardId),
    /// Prompt is being shown to sacrifice cards due to exceeding the
    /// limit for cards in play of this type. Player must sacrifice until they
    /// have the provided number of minions in the room.
    CardLimit(CardType, Option<CardSubtype>),
    /// Sacrifice a card to prevent up to `DamageAmount` damage. Will inspect
    /// the current incoming damage value and display only the lower of the two
    /// values.
    SacrificeToPreventDamage(CardId, DamageAmount),
    /// Sacrifice a card to prevent up to `CurseCount` curses. Will inspect
    /// the current incoming curse count and display only the lower of the two
    /// values.
    SacrificeToPreventCurses(CardId, CurseCount),
}

impl ButtonPromptContext {
    /// Looks up the card associated with this prompt, if any
    pub fn associated_card(&self) -> Option<CardId> {
        match self {
            Self::Card(id) => Some(*id),
            Self::SacrificeToPreventDamage(id, _) => Some(*id),
            Self::SacrificeToPreventCurses(id, _) => Some(*id),
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

/// Target game object for a [CardSelectorPrompt] to which cards must be
/// dragged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserPromptTarget {
    DiscardPile,
    Deck,
}

/// Describes which configurations of subjects for a [CardSelectorPrompt] are
/// valid and should allow the prompt to be exited.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserPromptValidation {
    /// User must select an exact quantity of cards.
    ExactlyCount(usize),
}

/// Describes the action which should be performed for a [CardSelectorPrompt] on
/// the `chosen_subjects` cards once the user submits their final choice.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BrowserPromptAction {
    /// Move the chosen subjects to the discard pile.
    DiscardCards,
}

/// A prompt which displays a selection of cards to the user and requests that
/// they drag cards to a target, e.g. in order to discard them from hand or
/// shuffle cards from their discard pile into their deck.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSelectorPrompt {
    /// Identifies the context for this prompt, i.e. why it is being shown to
    /// the user
    pub context: Option<PromptContext>,
    /// Cards which should be displayed in the browser and which have not
    /// been selected by dragging them to the target. Initially, this should
    /// contain all subject cards. As cards are dragged in the UI, they will be
    /// removed from this list and added to `chosen_subjects`.
    ///
    /// For example, this would contain cards that should be kept in hand during
    /// the 'discard to hand size' flow.
    pub unchosen_subjects: Vec<CardId>,
    /// Cards which have been selected, e.g. the cards that should be discarded
    /// when performing the 'discard to hand size' flow. This should initially
    /// be empty.
    pub chosen_subjects: Vec<CardId>,
    /// Target game object to which cards must be dragged.
    pub target: BrowserPromptTarget,
    /// Describes which configurations of subjects are valid and should allow
    /// the prompt to be exited.
    pub validation: BrowserPromptValidation,
    /// Describes the action which should be performed on the `chosen_subjects`
    /// cards once the user submits their final choice.
    pub action: BrowserPromptAction,
}

/// Action to take on cards which are *not* played via the [PlayCardBrowser].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UnplayedAction {
    None,
    Discard,
}

/// A browser shown to the user to allow them to play one or more cards from a
/// set of cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayCardBrowser {
    /// Identifies the context for this prompt, i.e. why it is being shown to
    /// the user
    pub context: Option<PromptContext>,
    /// Identifies the ability which caused this browser to be displayed.
    pub initiated_by: AbilityId,
    /// Identifies the choices of cards that the user can possibly play.
    pub cards: Vec<CardId>,
    /// Action to take on cards which are *not* played
    pub unplayed_action: UnplayedAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PromptChoiceLabel {
    Sacrifice,
    Return(ManaValue),
}

/// A specific card choice shown in a [ButtonPrompt].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptChoice {
    /// Effects of selecting this option
    pub effects: Vec<GameEffect>,
    /// Optionally, an anchor card for this prompt. If provided, the button will
    /// be rendered attached to this card in the game interface.
    pub anchor_card: Option<CardId>,
    /// A custom button label to associate with this choice. If not provided,
    /// the button's label will be derived from each of the [GameEffect]s
    /// concatenated using ", ".
    pub custom_label: Option<PromptChoiceLabel>,
}

impl PromptChoice {
    pub fn new() -> Self {
        Self { effects: vec![], anchor_card: None, custom_label: None }
    }

    pub fn effect(mut self, effect: GameEffect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn anchor_card(mut self, card_id: CardId) -> Self {
        self.anchor_card = Some(card_id);
        self
    }

    pub fn custom_label(mut self, label: PromptChoiceLabel) -> Self {
        self.custom_label = Some(label);
        self
    }

    /// Returns true if this prompt choice should be de-emphasized in the UI
    /// (e.g. rendered with a gray button).
    pub fn is_secondary(&self) -> bool {
        self.effects.iter().any(|effect| effect.is_secondary())
    }
}

/// Presents a choice to a user presented via buttons attached to specific cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonPrompt {
    /// Identifies the context for this prompt, i.e. why it is being shown to
    /// the user.
    pub context: Option<ButtonPromptContext>,
    /// Card actions for this prompt
    pub choices: Vec<PromptChoice>,
}

/// Possible types of prompts which might be displayed to a user during the
/// game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamePrompt {
    ButtonPrompt(ButtonPrompt),
    CardSelector(CardSelectorPrompt),
    PlayCardBrowser(PlayCardBrowser),
}

/// Possible actions in response to the [GamePrompt] currently being shown to a
/// user
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PromptAction {
    /// Select the option at the provided index in the current [ButtonPrompt].
    ButtonPromptSelect(usize),
    /// Submit the current selection in the current [CardSelectorPrompt].
    CardSelectorSubmit,
    /// Button to avoid playing a card when shown a 'Play Card' browser
    SkipPlayingCard,
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

#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RaidAction {
    /// Index position of the action to take in the current `RaidStep`.
    pub index: usize,
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
    UnveilCard(CardId),
    RemoveCurse,
    DispelEvocation,
    InitiateRaid(RoomId),
    LevelUpRoom(RoomId),
    SpendActionPoint,
    MoveSelectorCard(CardId),
    RaidAction(RaidAction),
    PromptAction(PromptAction),
    Undo,
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
            Self::UnveilCard(id) => f.debug_tuple("@UnveilCard").field(id).finish(),
            Self::RemoveCurse => write!(f, "@RemoveCurse"),
            Self::DispelEvocation => write!(f, "@DispelEvocation"),
            Self::InitiateRaid(arg0) => f.debug_tuple("@InitiateRaid").field(arg0).finish(),
            Self::LevelUpRoom(arg0) => f.debug_tuple("@LevelUpRoom").field(arg0).finish(),
            Self::SpendActionPoint => write!(f, "@SpendActionPoint"),
            Self::MoveSelectorCard(id) => f.debug_tuple("@MoveCard").field(id).finish(),
            Self::RaidAction(action) => f.debug_tuple("@RaidAction").field(&action.index).finish(),
            Self::PromptAction(prompt) => write!(f, "@{prompt:?}"),
            Self::Undo => write!(f, "@Undo"),
        }
    }
}
