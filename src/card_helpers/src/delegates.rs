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

use game_data::delegate_data::{
    Delegate, EventDelegate, Flag, MutationFn, QueryDelegate, RaidEvent, RequirementFn,
    ShieldCardInfo, TransformationFn,
};
use game_data::primitives::{CardId, ManaValue, RaidId, ShieldValue};

pub fn mana_cost(
    requirement: RequirementFn<CardId>,
    transformation: TransformationFn<CardId, Option<ManaValue>>,
) -> Delegate {
    Delegate::ManaCost(QueryDelegate { requirement, transformation })
}

pub fn sanctum_access_count(
    requirement: RequirementFn<RaidId>,
    transformation: TransformationFn<RaidId, u32>,
) -> Delegate {
    Delegate::SanctumAccessCount(QueryDelegate { requirement, transformation })
}

pub fn shield_value(
    requirement: RequirementFn<ShieldCardInfo>,
    transformation: TransformationFn<ShieldCardInfo, ShieldValue>,
) -> Delegate {
    Delegate::ShieldValue(QueryDelegate { requirement, transformation })
}

pub fn on_minion_approached(
    requirement: RequirementFn<RaidEvent<CardId>>,
    mutation: MutationFn<RaidEvent<CardId>>,
) -> Delegate {
    Delegate::ApproachMinion(EventDelegate { requirement, mutation })
}

pub fn on_raid_access_start(
    requirement: RequirementFn<RaidEvent<()>>,
    mutation: MutationFn<RaidEvent<()>>,
) -> Delegate {
    Delegate::RaidAccessStart(EventDelegate { requirement, mutation })
}

pub fn on_raid_successful(
    requirement: RequirementFn<RaidEvent<()>>,
    mutation: MutationFn<RaidEvent<()>>,
) -> Delegate {
    Delegate::RaidSuccess(EventDelegate { requirement, mutation })
}

pub fn can_raid_access_cards(
    requirement: RequirementFn<RaidEvent<()>>,
    transformation: TransformationFn<RaidEvent<()>, Flag>,
) -> Delegate {
    Delegate::CanRaidAccessCards(QueryDelegate { requirement, transformation })
}
