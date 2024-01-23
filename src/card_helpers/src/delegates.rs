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
    AbilityId, CardId, InitiatedBy, ManaValue, RaidId, Resonance, ShieldValue, Side,
};
use enumset::EnumSet;
use game_data::continuous_visual_effect::ContinuousDisplayEffect;
use game_data::delegate_data::{
    AccessEvent, CardPlayed, CardStatusMarker, EventDelegate, GameDelegate, MutationFn,
    QueryDelegate, RaidEvent, RaidOutcome, RequirementFn, Scope, ShieldCardInfo, TransformationFn,
};
use game_data::flag_data::{AbilityFlag, Flag};
use game_data::game_state::GameState;
use game_data::raid_data::{MinionDefeated, PopulateAccessPromptSource};

/// A [TransformationFn] which invokes [Flag::allow] to enable an action.
pub fn allow<T>(_: &GameState, _: Scope, _: &T, flag: Flag) -> Flag {
    flag.allow()
}

/// A [TransformationFn] which invokes [Flag::disallow] to prevent an action.
pub fn disallow<T>(_: &GameState, _: Scope, _: &T, flag: Flag) -> Flag {
    flag.disallow()
}

/// A [TransformationFn] which invokes [AbilityFlag::disallow] to prevent an
/// action.
pub fn disallow_ability<T>(_: &GameState, s: Scope, _: &T, flag: AbilityFlag) -> AbilityFlag {
    flag.disallow(s.ability_id())
}

pub fn mana_cost(
    requirement: RequirementFn<CardId>,
    transformation: TransformationFn<CardId, Option<ManaValue>>,
) -> GameDelegate {
    GameDelegate::ManaCost(QueryDelegate { requirement, transformation })
}

pub fn sanctum_access_count(
    requirement: RequirementFn<RaidId>,
    transformation: TransformationFn<RaidId, u32>,
) -> GameDelegate {
    GameDelegate::SanctumAccessCount(QueryDelegate { requirement, transformation })
}

pub fn vault_access_count(
    requirement: RequirementFn<RaidId>,
    transformation: TransformationFn<RaidId, u32>,
) -> GameDelegate {
    GameDelegate::VaultAccessCount(QueryDelegate { requirement, transformation })
}

pub fn shield_value(
    requirement: RequirementFn<ShieldCardInfo>,
    transformation: TransformationFn<ShieldCardInfo, ShieldValue>,
) -> GameDelegate {
    GameDelegate::ShieldValue(QueryDelegate { requirement, transformation })
}

pub fn on_played(
    requirement: RequirementFn<CardPlayed>,
    mutation: MutationFn<CardPlayed>,
) -> GameDelegate {
    GameDelegate::PlayCard(EventDelegate { requirement, mutation })
}

pub fn on_will_populate_summon_prompt(
    requirement: RequirementFn<RaidEvent<CardId>>,
    mutation: MutationFn<RaidEvent<CardId>>,
) -> GameDelegate {
    GameDelegate::WillPopulateSummonPrompt(EventDelegate { requirement, mutation })
}

pub fn on_minion_summoned(
    requirement: RequirementFn<CardId>,
    mutation: MutationFn<CardId>,
) -> GameDelegate {
    GameDelegate::SummonMinion(EventDelegate { requirement, mutation })
}

pub fn on_minion_approached(
    requirement: RequirementFn<RaidEvent<CardId>>,
    mutation: MutationFn<RaidEvent<CardId>>,
) -> GameDelegate {
    GameDelegate::ApproachMinion(EventDelegate { requirement, mutation })
}

pub fn on_minion_encountered(
    requirement: RequirementFn<CardId>,
    mutation: MutationFn<CardId>,
) -> GameDelegate {
    GameDelegate::EncounterMinion(EventDelegate { requirement, mutation })
}

pub fn on_minion_defeated(
    requirement: RequirementFn<MinionDefeated>,
    mutation: MutationFn<MinionDefeated>,
) -> GameDelegate {
    GameDelegate::MinionDefeated(EventDelegate { requirement, mutation })
}

pub fn on_raid_access_start(
    requirement: RequirementFn<RaidEvent<()>>,
    mutation: MutationFn<RaidEvent<()>>,
) -> GameDelegate {
    GameDelegate::RaidAccessStart(EventDelegate { requirement, mutation })
}

pub fn on_will_populate_access_prompt(
    requirement: RequirementFn<AccessEvent<PopulateAccessPromptSource>>,
    mutation: MutationFn<AccessEvent<PopulateAccessPromptSource>>,
) -> GameDelegate {
    GameDelegate::WillPopulateAccessPrompt(EventDelegate { requirement, mutation })
}

pub fn on_card_access(
    requirement: RequirementFn<AccessEvent<CardId>>,
    mutation: MutationFn<AccessEvent<CardId>>,
) -> GameDelegate {
    GameDelegate::CardAccess(EventDelegate { requirement, mutation })
}

pub fn on_card_razed(
    requirement: RequirementFn<AccessEvent<CardId>>,
    mutation: MutationFn<AccessEvent<CardId>>,
) -> GameDelegate {
    GameDelegate::RazeCard(EventDelegate { requirement, mutation })
}

pub fn on_custom_access_end(
    requirement: RequirementFn<InitiatedBy>,
    mutation: MutationFn<InitiatedBy>,
) -> GameDelegate {
    GameDelegate::CustomAccessEnd(EventDelegate { requirement, mutation })
}

pub fn on_ability_will_end_raid(
    requirement: RequirementFn<RaidEvent<AbilityId>>,
    mutation: MutationFn<RaidEvent<AbilityId>>,
) -> GameDelegate {
    GameDelegate::AbilityWillEndRaid(EventDelegate { requirement, mutation })
}

pub fn on_raid_end(
    requirement: RequirementFn<RaidEvent<RaidOutcome>>,
    mutation: MutationFn<RaidEvent<RaidOutcome>>,
) -> GameDelegate {
    GameDelegate::RaidEnd(EventDelegate { requirement, mutation })
}

pub fn on_raid_successful(
    requirement: RequirementFn<RaidEvent<()>>,
    mutation: MutationFn<RaidEvent<()>>,
) -> GameDelegate {
    GameDelegate::RaidSuccess(EventDelegate { requirement, mutation })
}

pub fn on_will_draw_cards(
    requirement: RequirementFn<Side>,
    mutation: MutationFn<Side>,
) -> GameDelegate {
    GameDelegate::WillDrawCards(EventDelegate { requirement, mutation })
}

pub fn on_query_resonance(
    requirement: RequirementFn<CardId>,
    transformation: TransformationFn<CardId, EnumSet<Resonance>>,
) -> GameDelegate {
    GameDelegate::Resonance(QueryDelegate { requirement, transformation })
}

pub fn on_query_card_status_markers(
    requirement: RequirementFn<CardId>,
    transformation: TransformationFn<CardId, Vec<CardStatusMarker>>,
) -> GameDelegate {
    GameDelegate::CardStatusMarkers(QueryDelegate { requirement, transformation })
}

pub fn can_play_card(
    requirement: RequirementFn<CardId>,
    transformation: TransformationFn<CardId, Flag>,
) -> GameDelegate {
    GameDelegate::CanPlayCard(QueryDelegate { requirement, transformation })
}

pub fn can_summon(
    requirement: RequirementFn<CardId>,
    transformation: TransformationFn<CardId, Flag>,
) -> GameDelegate {
    GameDelegate::CanSummon(QueryDelegate { requirement, transformation })
}

pub fn can_score_accessed_card(
    requirement: RequirementFn<AccessEvent<CardId>>,
    transformation: TransformationFn<AccessEvent<CardId>, Flag>,
) -> GameDelegate {
    GameDelegate::CanScoreAccessedCard(QueryDelegate { requirement, transformation })
}

pub fn can_covenant_score_scheme(
    requirement: RequirementFn<CardId>,
    transformation: TransformationFn<CardId, AbilityFlag>,
) -> GameDelegate {
    GameDelegate::CanCovenantScoreScheme(QueryDelegate { requirement, transformation })
}

pub fn status_markers(
    requirement: RequirementFn<CardId>,
    transformation: TransformationFn<CardId, Vec<CardStatusMarker>>,
) -> GameDelegate {
    GameDelegate::CardStatusMarkers(QueryDelegate { requirement, transformation })
}

pub fn continuous_display_effect(
    requirement: RequirementFn<CardId>,
    transformation: TransformationFn<CardId, ContinuousDisplayEffect>,
) -> GameDelegate {
    GameDelegate::ContinuousDisplayEffect(QueryDelegate { requirement, transformation })
}
