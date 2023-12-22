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
    AbilityId, CardId, CardPlayId, CardType, HasAbilityId, InitiatedBy, ManaValue, RoomId,
};
use serde::{Deserialize, Serialize};

use crate::delegate_data::{AbilityActivated, CardPlayed};
use crate::game_actions::ButtonPromptContext;
use crate::game_effect::GameEffect;

/// Stores data associated with a GamePrompt for later use in rendering that
/// prompt.
///
/// This essentially exists because of serialization: normally we would just
/// have prompts be rendered by invoking a closure which captures relevant
/// state. Unfortunately there is no way to serialize such a closure to JSON, so
/// we instead store the relevant context in this enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptData {
    None,
    Index(u32),
    Room(RoomId),
    Card(CardId),
    Cards(Vec<CardId>),
    CardPlay(CardPlayed),
    AbilityActivation(AbilityActivated),
    CardPlayId(CardPlayId),
}

/// Identifies where an ability prompt came from.
///
/// This is used as part of the prompt callback system to register an ability as
/// having a prompt. The system later calls that ability back via its delegate
/// to render that prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityPromptSource {
    pub ability_id: AbilityId,
    pub data: PromptData,
}

impl HasAbilityId for AbilityPromptSource {
    fn ability_id(&self) -> AbilityId {
        self.ability_id
    }
}

/// Tuple of [GamePrompt] and [AbilityPromptSource], used to store a prompt and
/// keep track of which ability generated it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEntry {
    pub prompt: GamePrompt,
    pub source: Option<AbilityPromptSource>,
}

/// A standard stack data structure for storing [GamePrompt]s.
///
/// Instead of directly manipulating this struct, you should always use the
/// functions in the `mutations::prompts` module instead. See the documentation
/// there, especially `prompts::push()`, for information about how the prompt
/// system works.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptStack {
    pub stack: Vec<PromptEntry>,
}

/// Describes the reason why a prompt is being shown
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
    /// Move a card to the top of the vault
    MoveToTopOfVault,
    /// Move cards to the top of the vault in a chosen order
    ReorderTopOfVault,
    /// Shuffle cards into the vault
    ShuffleIntoVault,
}

/// Target game object for a [CardSelectorPrompt] to which cards must be
/// dragged.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BrowserPromptTarget {
    DiscardPile,
    DeckTop,
    DeckShuffled,
}

/// Describes which configurations of subjects for a [CardSelectorPrompt] are
/// valid and should allow the prompt to be exited.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserPromptValidation {
    /// User must select an exact quantity of cards.
    ExactlyCount(usize),
    /// User must select at most this many cards.
    LessThanOrEqualTo(usize),
    /// User must move all subject cards
    AllSubjects,
}

/// A prompt which displays a selection of cards to the user and requests that
/// they drag cards to a target, e.g. in order to discard them from hand or
/// shuffle cards from their discard pile into their deck.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSelectorPrompt {
    /// Identifies the context for this prompt, i.e. why it is being shown to
    /// the user
    pub context: Option<PromptContext>,
    /// Identifies the source which caused this selector to be displayed.
    pub initiated_by: InitiatedBy,
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
    /// Target game object to which cards card being moved.
    pub target: BrowserPromptTarget,
    /// Describes which configurations of subjects are valid and should allow
    /// the prompt to be exited.
    pub validation: Option<BrowserPromptValidation>,
    /// If true, the player seeing this prompt can rearrange the cards within
    /// the `target` position.
    pub can_reorder: bool,
}

/// Action to take on cards which are *not* played via the [PlayCardBrowser].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UnplayedAction {
    None,
    Discard,
}

/// Possible points of origin for cards being played.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum FromZone {
    Hand,
    Discard,
    Deck,
    Banished,
}

/// A browser shown to the user to allow them to play one or more cards from a
/// set of cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayCardBrowser {
    /// Identifies the context for this prompt, i.e. why it is being shown to
    /// the user
    pub context: Option<PromptContext>,
    /// Zone of origin for the cards being played.
    pub from_zone: FromZone,
    /// Identifies the ability which caused this browser to be displayed.
    pub initiated_by: AbilityId,
    /// Identifies the choices of cards that the user can possibly play.
    pub cards: Vec<CardId>,
    /// Action to take on cards which are *not* played
    pub unplayed_action: UnplayedAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromptChoiceLabel {
    Play,
    Sacrifice,
    Prevent,
    Return,
    ReturnForCost(ManaValue),
    Occupant,
    Defender,
    PayActionAccessAnother,
    CardType(CardType),
    Select,
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

    pub fn new_continue() -> Self {
        Self { effects: vec![GameEffect::Continue], anchor_card: None, custom_label: None }
    }

    pub fn effect(mut self, effect: GameEffect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn effect_optional(mut self, effect: Option<GameEffect>) -> Self {
        if let Some(e) = effect {
            self.effects.push(e);
        }
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

/// Reason why a [RoomSelectorPrompt] is being shown
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoomSelectorPromptContext {
    Access,
}

/// Mutation to apply to the room chosen via a [RoomSelectorPrompt].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoomSelectorPromptEffect {
    ChangeRaidTarget,
}

/// Shows a prompt to pick a target room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSelectorPrompt {
    pub initiated_by: AbilityId,
    pub effect: RoomSelectorPromptEffect,
    pub valid_rooms: Vec<RoomId>,
    pub context: Option<RoomSelectorPromptContext>,
}

/// Possible types of prompt_ui which might be displayed to a user during the
/// game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamePrompt {
    ButtonPrompt(ButtonPrompt),
    CardSelector(CardSelectorPrompt),
    PlayCardBrowser(PlayCardBrowser),

    /// Prompt which lets a player activate abilities when they otherwise could
    /// not
    PriorityPrompt,

    /// Prompt to pick a room
    RoomSelector(RoomSelectorPrompt),
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
    /// Select a room via a [RoomSelectorPrompt].
    RoomPromptSelect(RoomId),
}
