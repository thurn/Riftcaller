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

use std::collections::HashMap;

use core_data::game_primitives::{AbilityId, CardId, InitiatedBy, RoomId, Side};
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::delegate_data::{AccessEvent, RaidEvent, UsedWeapon};
use crate::game_actions::CardTarget;
use crate::game_state::TurnData;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum AbilityActivationType {
    /// Activated ability had an action point cost and thus counts as a full
    /// game action.
    GameAction,
    /// Activated ability did not have an action point cost.
    FreeAction,
}

/// Information about the context in which an ability was activated
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AbilityActivation {
    /// ID of activated ability
    pub ability_id: AbilityId,
    /// Target for the ability, if any
    pub target: CardTarget,
    /// Whether the ability was activated as a full game action. This is mostly
    /// used to determine whether "play only as your first action" cards can
    /// still be played.
    pub activation_type: AbilityActivationType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum CardChoice {
    CardId(CardId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardChoiceEvent {
    pub ability_id: AbilityId,
    pub choice: CardChoice,
}

/// Records a single event which happened during this game.
#[derive(Debug, Clone, Serialize, Deserialize, EnumKind)]
#[enum_kind(HistoryEventKind)]
pub enum HistoryEvent {
    /// Mana was gained via the standard game action
    GainManaAction,
    /// The 'draw card' standard game action was taken by the [Side] player.
    DrawCardAction(Side),
    /// Curse removed via the standard game action
    RemoveCurseAction,
    /// Evocation destroyed via the standard game action
    DispelEvocationAction,
    /// A card was played, either via the standard game action or initiated by
    /// an ability of another card.
    PlayCard(CardId, CardTarget, InitiatedBy),
    /// A card ability was activated
    ActivateAbility(AbilityActivation),
    /// A face-down card has been summoned.
    SummonProject(CardId),
    /// A room was progressed some number of times via the standard game action
    CardProgressAction(RoomId),
    /// A raid was started, either via a card effect or the standard game action
    RaidBegin(RaidEvent<InitiatedBy>),
    /// A minion has been summoned during a raid.
    MinionSummon(RaidEvent<CardId>),
    /// A minion has been approached during a raid.
    MinionApproach(RaidEvent<CardId>),
    /// A minion has been encountered during a raid.
    MinionEncounter(RaidEvent<CardId>),
    /// A weapon has been used on minion
    UseWeapon(RaidEvent<UsedWeapon>),
    /// A minion's combat ability has triggered
    MinionCombatAbility(RaidEvent<CardId>),
    /// A card's raze ability has been activated during a raid access phase
    RazeAccessedCard(AccessEvent<CardId>),
    /// A card has been scored during a raid access phase
    ScoreAccessedCard(AccessEvent<CardId>),
    /// A raid ended in success.
    RaidSuccess(RaidEvent<()>),
    /// A raid ended in failure.
    RaidFailure(RaidEvent<()>),
}

impl HistoryEvent {
    /// Returns the [HistoryEventKind] for this event
    pub fn kind(&self) -> HistoryEventKind {
        self.into()
    }
}

/// Tuple of [TurnData] and [HistoryEvent].
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    turn: TurnData,
    event: HistoryEvent,
}

static DEFAULT_COUNTERS: HistoryCounters = HistoryCounters {
    cards_drawn: 0,
    cards_drawn_via_abilities: 0,
    curses_received: 0,
    damage_received: 0,
};

/// Counters for events that happen during a given turn. Each player has their
/// own set of counters for game events.
///
/// All counters default to 0 at start of turn. History counters should always
/// be updated as the final step in any game mutation, for example the "draw
/// cards" event should draw the cards and fire related game events *before*
/// updating the counter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryCounters {
    /// Cards drawn so far this turn by this player. This records the actual
    /// number of cards drawn, e.g. if a player attempts to draw from an empty
    /// deck no draw is recorded.
    pub cards_drawn: u32,
    /// Cards drawn this turn via card abilities by this player
    pub cards_drawn_via_abilities: u32,
    /// Number of curses received this turn, only valid for the Champion player.
    pub curses_received: u32,
    /// Amount of damage received this turn, only valid for the Champion player.
    pub damage_received: u32,
}

/// History of events which have happened during this game.
///
/// This operates via a two-phase system where history entries are collected
/// during action resolution, but are not immediately visible in the general
/// history until they are finalized by calling [Self::write_events], usually as
/// the final step of any game action. This helps avoid confusion where events
/// added during the *current* action appear in history, which is typically not
/// desired.
#[serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameHistory {
    current: Vec<HistoryEntry>,
    #[serde_as(as = "Vec<(_, _)>")]
    entries: HashMap<TurnData, Vec<HistoryEvent>>,
    #[serde_as(as = "Vec<(_, _)>")]
    overlord_counters: HashMap<TurnData, HistoryCounters>,
    #[serde_as(as = "Vec<(_, _)>")]
    champion_counters: HashMap<TurnData, HistoryCounters>,
}

impl GameHistory {
    /// Returns history events in the provided turn, *before* the current game
    /// event.
    pub fn for_turn(&self, turn: TurnData) -> impl Iterator<Item = &HistoryEvent> {
        self.entries.get(&turn).into_iter().flatten()
    }

    /// Returns a  reference to the [HistoryCounters] entry for the provided
    /// turn.
    pub fn counters_for_turn(&self, turn: TurnData, side: Side) -> &HistoryCounters {
        match side {
            Side::Overlord => self.overlord_counters.get(&turn).unwrap_or(&DEFAULT_COUNTERS),
            Side::Champion => self.champion_counters.get(&turn).unwrap_or(&DEFAULT_COUNTERS),
        }
    }

    /// Returns a mutable reference to the [HistoryCounters] entry for the
    /// provided turn.
    pub fn counters_for_turn_mut(&mut self, turn: TurnData, side: Side) -> &mut HistoryCounters {
        match side {
            Side::Overlord => self.overlord_counters.entry(turn).or_default(),
            Side::Champion => self.champion_counters.entry(turn).or_default(),
        }
    }

    /// Adds a new history entry to the 'current events' buffer. Events do
    /// not appear in the [Self::for_turn] history until they are finalized by
    /// calling [Self::write_events], which typically happens as the last step
    /// in processing a game action.
    pub fn add_event(&mut self, turn: TurnData, event: HistoryEvent) {
        self.current.push(HistoryEntry { turn, event })
    }

    /// Writes all stored history events to the game history and clears the
    /// 'current events' buffer.
    pub fn write_events(&mut self) {
        for entry in &self.current {
            self.entries.entry(entry.turn).or_default().push(entry.event.clone());
        }

        self.current.clear();
    }
}
