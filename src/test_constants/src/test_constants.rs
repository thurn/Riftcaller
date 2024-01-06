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

//! Standard values used in unit tests.

use core_data::adventure_primitives::Coins;
use core_data::game_primitives::{HealthValue, ManaValue, RaidId, Resonance, RoomId};
use protos::riftcaller::RoomIdentifier;

/// The title returned for hidden cards
pub const HIDDEN_CARD: &str = "Hidden Card";

/// [RoomId] used by default for targeting
pub const ROOM_ID: RoomId = RoomId::RoomA;

/// Client equivalent of [ROOM_ID].
pub const CLIENT_ROOM_ID: RoomIdentifier = RoomIdentifier::RoomA;

/// Default Raid ID to use during testing
pub const RAID_ID: RaidId = RaidId(1);

/// Default mana for players in a test game if not otherwise specified
pub const STARTING_MANA: ManaValue = 999;

pub const STARTING_COINS: Coins = Coins(999);

pub const SPELL_COST: ManaValue = 1;

pub const MINION_COST: ManaValue = 3;

pub const WEAPON_COST: ManaValue = 3;

pub const ARTIFACT_COST: ManaValue = 1;

pub const SUMMON_PROJECT_COST: ManaValue = 3;

pub const EVOCATION_COST: ManaValue = 3;

pub const ALLY_COST: ManaValue = 3;

pub const RAZE_COST: ManaValue = 2;

pub const MANA_STORED: ManaValue = 10;

pub const MANA_TAKEN: ManaValue = 2;

pub const MINION_HEALTH: HealthValue = 5;

pub const TEST_RESONANCE: Resonance = Resonance::Infernal;
