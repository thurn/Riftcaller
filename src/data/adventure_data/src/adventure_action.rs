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

use core_data::adventure_primitives::{NarrativeChoiceIndex, TilePosition};
use game_data::card_name::CardVariant;
use serde::{Deserialize, Serialize};

use crate::adventure::NarrativeEventStep;

/// Vector index for locating effects to apply within a given narrative
/// encounter.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum NarrativeEffectIndex {
    Cost(usize),
    Reward(usize),
}

/// Actions which can be taken for the 'adventure' game mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum AdventureAction {
    /// Transition an adventure to the 'completed' state and display the
    /// adventure summary screen. Can be followed by
    /// `UserAction::LeaveAdventure` to completely exit the adventure.
    AbandonAdventure,
    /// Visit the entity at a given adventure tile position
    VisitTileEntity(TilePosition),
    /// Stop visiting a tile entity
    EndVisit,
    /// Draft the card at the indicated index on the draft screen
    DraftCard(usize),
    /// Draft the purchase at the indicated index on the current shop screen
    BuyCard(usize),
    /// Jump to one of the steps on a narrative choice screen
    SetNarrativeStep(NarrativeEventStep),
    /// Apply one effect of a narrative event. This can result in playing an
    /// animation or opening another screen (such as showing a 'draft' panel).
    ApplyNarrativeEffect(NarrativeChoiceIndex, NarrativeEffectIndex),
    /// Ends the current narrative event screen
    EndNarrativeEvent,
    /// Apply the current deck card effect to a named card.
    ///
    /// The current screen must be an 'ApplyDeckEffect' screen, and the effect
    /// to use is queried from that screen's state.
    ApplyDeckCardEffect(CardVariant),
    /// Stop showing the deck card effects editor screen.
    CloseDeckCardEffects,
}
