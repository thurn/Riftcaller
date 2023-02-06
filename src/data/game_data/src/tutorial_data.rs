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

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::card_name::CardName;
use crate::game_actions::CardTarget;
use crate::primitives::{Milliseconds, RoomId, Side};

/// Displays an arrow pointing to a specific piece of the user interface
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum TooltipAnchor {
    RaidRoom(RoomId),
    GainMana,
    DrawCard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeechBubble {
    pub text: String,
    pub side: Side,
    pub delay: Milliseconds,

    // Whether this text should repeat every turn until an action is matched
    pub recurring: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tooltip {
    pub text: String,
    pub anchor: TooltipAnchor,
    pub delay: Milliseconds,

    // Whether this tooltip should repeat every turn until an action is matched
    pub recurring: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Toast {
    pub text: String,
    pub delay: Milliseconds,
    pub hide_after: Option<Milliseconds>,
}

/// Content which can be displayed to the user during the game tutorial
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TutorialDisplay {
    // Show text and an arrow pointing to a specific UI element
    Tooltip(Tooltip),

    // Display text spoken by a specific player
    SpeechBubble(SpeechBubble),

    // Pop up a general help message to the user
    Toast(Toast),
}

impl TutorialDisplay {
    pub fn recurring(&self) -> bool {
        match self {
            TutorialDisplay::Tooltip(tooltip) => tooltip.recurring,
            TutorialDisplay::SpeechBubble(bubble) => bubble.recurring,
            TutorialDisplay::Toast(_) => false,
        }
    }
}

/// State of the in-game tutorial
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GameTutorialState {
    pub data: TutorialData,

    /// Current position within the `tutorial::STEPS` vector
    pub index: usize,

    /// Action indices which have been seen in the current tutorial step via
    /// `Tutorial::AwaitPlayerActions`
    pub action_indices_seen: HashSet<usize>,

    /// Tutorial content to show to the user
    pub display: Vec<TutorialDisplay>,
}

/// Opponent actions during the tutorial which are scripted to occur
#[derive(Debug)]
pub enum TutorialOpponentAction {
    DrawCard,
    PlayCard(CardName, CardTarget),
    GainMana,
    InitiateRaid(RoomId),
    LevelUpRoom(RoomId),
    UseWeapon { weapon: CardName, target: CardName },
    ScoreAccessedCard(CardName),
    EndRaid,
}

/// Matches against game states to trigger tutorial messages
#[derive(Debug)]
pub enum TutorialGameStateTrigger {
    HandContainsCard(Side, CardName),
}

/// Matches against user actions to trigger tutorial messages
#[derive(Debug)]
pub enum TutorialTrigger {
    DrawCardAction,
    PlayAnyCard,
    PlayCard(CardName, CardTarget),
    GainManaAction,
    InitiateRaid(RoomId),
    LevelUpRoom(RoomId),
    UseWeapon { weapon: CardName, target: CardName },
    UseNoWeapon,
    ScoreAccessedCard(CardName),
    SuccessfullyEndRaid,
}

/// Declarative description of events & actions during the tutorial. The
/// tutorial system hooks into games based on the `GameConfiguration::tutorial`
/// flag and mutates their `GameState` before user or AI `GameAction`s get
/// processed.
#[derive(Debug)]
pub enum TutorialStep {
    /// Causes a player to keep their opening hand
    KeepOpeningHand(Side),

    /// Overwrites the contents of a player's hand with new cards. All cards
    /// must exist already in the player's deck. Also causes this player to
    /// select the 'keep opening hand' prompt option.
    SetHand(Side, Vec<CardName>),

    /// Moves the leader card for a player into play
    SetLeaderInPlay(Side),

    /// Sets the top card of a player's deck to contain specific cards. The
    /// cards must exist already in the player's deck.
    ///
    /// Later items in the vector are closer to the top of the deck.
    SetTopOfDeck(Side, Vec<CardName>),

    /// Cause the opponent to perform the indicated game actions, bypassing the
    /// AI. The last action is repeated if the opponent's turn comes again
    /// without advancing the tutorial state.
    OpponentAction(TutorialOpponentAction),

    /// Cause the opponent to perform the indicated game action repeatedly
    /// whenever their turn comes up, if no OpponentAction is specified.
    DefaultOpponentAction(TutorialOpponentAction),

    /// Wait for the user to perform all of the indicated actions before
    /// advancing to the next tutorial step. Other game actions are still
    /// allowed, but they won't cause the tutorial to advance.
    AwaitTriggers(Vec<TutorialTrigger>),

    /// Wait for a specific game state before continuing
    AwaitGameState(TutorialGameStateTrigger),

    /// Provide tutorial information to show to the user until any player action
    /// is taken.
    Display(Vec<TutorialDisplay>),

    /// Adds global modifier cards
    AddGameModifiers(Vec<CardName>),

    /// Removes global modifier cards
    RemoveGameModifiers(Vec<CardName>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum TutorialMessageKey {
    DeckEditor,
    PlayAbilityCard,
    PlayInfernalWeapon,
}

#[derive(Debug)]
pub struct TutorialMessageTrigger {
    pub key: TutorialMessageKey,
    pub trigger: TutorialTrigger,
    pub display: Vec<TutorialDisplay>,
}

#[derive(Debug)]
pub struct TutorialSequence {
    pub steps: Vec<TutorialStep>,
    /// Messages which are displayed when some matching game action is taken
    pub messages: Vec<TutorialMessageTrigger>,
}

/// Data model for the player's progress through the game's tutorial
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TutorialData {
    skip_all: bool,
    seen: HashSet<TutorialMessageKey>,
}

impl TutorialData {
    /// New default instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Avoid displaying any tutorial messages
    pub fn skip_all(mut self, skip_all: bool) -> Self {
        self.skip_all = skip_all;
        self
    }

    /// Returns true if the user has seen the tutorial message with the given
    /// key.
    pub fn has_seen(&self, key: TutorialMessageKey) -> bool {
        if self.skip_all {
            true
        } else {
            self.seen.contains(&key)
        }
    }

    /// Record that the user has seen the tutorial message with the given key
    pub fn mark_seen(&mut self, key: TutorialMessageKey) -> bool {
        self.seen.insert(key)
    }
}
