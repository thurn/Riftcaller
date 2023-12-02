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

use core_data::game_primitives::{AbilityId, CardId, GameObjectId, InitiatedBy, RoomId, Side};

use crate::game_state::GameState;
use crate::prompt_data::PromptChoice;
use crate::special_effects::SpecialEffect;

/// Indicates one game object targeted another with an effect.
///
/// Typically represented in animation as a projectile being fired.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TargetedInteraction {
    pub source: GameObjectId,
    pub target: GameObjectId,
}

/// Represents a change to the state of the game which should be translated
/// into a client animation
#[derive(Debug, Clone)]
pub enum GameAnimation {
    /// Indicates that a player's turn has started
    StartTurn(Side),
    /// A player has played a card face-up.
    PlayCard(Side, CardId),
    /// A request to play one or more visual or audio effects
    CustomEffects(Vec<SpecialEffect>),
    /// A player has activated an ability of a card
    AbilityActivated(Side, AbilityId),
    /// An ability of a card has triggered an effect
    AbilityTriggered(AbilityId, Vec<SpecialEffect>),
    /// One or more cards have been drawn by the [Side] player.
    DrawCards(Side, Vec<CardId>),
    /// A player has shuffled cards into their deck
    ShuffleIntoDeck,
    /// A project card has been turned face-up via the summon action
    SummonProject(CardId),
    /// A minion card has been turned face-up.
    SummonMinion(CardId),
    /// A minion card has been turned face-down.
    UnsummonMinion(CardId),
    /// A face-down card has been revealed to the Champion
    RevealCard(CardId),
    /// The Overlord has progressed a room
    ProgressRoom(RoomId, InitiatedBy),
    /// The Champion has initiated a raid on a room
    InitiateRaid(RoomId, InitiatedBy),
    /// Interaction between two cards during raid combat.
    CombatInteraction(TargetedInteraction),
    /// The Champion has accessed the specified cards in the Sanctum
    AccessSanctumCards(Vec<CardId>),
    /// A player has scored a card
    ScoreCard(Side, CardId),
    /// The game has ended and the indicated player has won
    GameOver(Side),
    /// Card selection browser has completed
    BrowserSubmitted,
    /// Show a 'play card' browser to play one of the indicated cards.
    ShowPlayCardBrowser(Vec<CardId>),
}

/// A step in the animation process
#[derive(Debug, Clone)]
pub struct AnimationStep {
    pub snapshot: GameState,
    pub update: GameAnimation,
}

/// Standard enum used by APIs to configure their update tracking behavior.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AnimationState {
    /// Game updates should not be tracked by the receiver
    Ignore,
    /// Game updates should be tracked by the receiver
    Track,
}

/// Tracks game mutations for a game action.
///
/// Some game state changes in Riftcaller require custom animations in the UI in
/// order to communicate their effects clearly. In order to implement the
/// animation system, code which mutates game state can also call
/// [GameState::add_animation] and provide a [GameAnimation] to record the
/// action they took. The way this process works is that a snapshot of the game
/// state is stored (to capture any mutations that occurred *before* the
/// animation), and then the update is stored. During the animation process, the
/// stored snapshots and [GameAnimation]s are played back sequentially.
///
/// Many types of state changes are handled automatically by the game state
/// snapshot system, so appending an update is only needed for custom
/// animations. For example the system will correctly detect and animate a card
/// which has moved to a new position.
#[derive(Debug, Clone)]
pub struct AnimationTracker {
    /// Used to globally disable or enable update tracking
    pub state: AnimationState,
    /// List of update steps with snapshots of the game state
    pub steps: Vec<AnimationStep>,
    /// Most recent user prompt response, if any.
    ///
    /// Used to show a speech bubble to the opponent with the result of prompt
    /// choices.gam
    pub last_prompt_response: Option<(Side, PromptChoice)>,
}

impl Default for AnimationTracker {
    fn default() -> Self {
        Self { state: AnimationState::Ignore, steps: vec![], last_prompt_response: None }
    }
}

impl AnimationTracker {
    pub fn new(updates: AnimationState) -> Self {
        Self { state: updates, steps: vec![], last_prompt_response: None }
    }
}
