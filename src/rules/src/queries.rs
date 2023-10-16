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

//! Core functions for querying the current state of a game

use anyhow::Result;
use constants::game_constants;
use game_data::card_definition::{AbilityType, CardStats, TargetRequirement};
use game_data::card_state::{CardPosition, CardState};
use game_data::delegate_data::{
    AbilityManaCostQuery, ActionCostQuery, BaseAttackQuery, BreachValueQuery, HealthValueQuery,
    ManaCostQuery, MaximumHandSizeQuery, RazeCostQuery, SanctumAccessCountQuery, ShieldCardInfo,
    ShieldValueQuery, StartOfTurnActionsQuery, VaultAccessCountQuery,
};
use game_data::game_actions::{CardTarget, CardTargetKind, GamePrompt};
use game_data::game_state::GameState;
use game_data::primitives::{
    AbilityId, ActionCount, AttackValue, BreachValue, CardId, CardType, HealthValue, ItemLocation,
    ManaValue, RazeCost, RoomId, RoomLocation, ShieldValue, Side,
};
use game_data::raid_data::{RaidData, RaidState, RaidStatus, RaidStep};

use crate::{dispatch, CardDefinitionExt};

/// Obtain the [CardStats] for a given card
pub fn stats(game: &GameState, card_id: CardId) -> &CardStats {
    &crate::get(game.card(card_id).variant).config.stats
}

/// Returns the mana cost for a given card.
///
/// - For minions, this is the summon cost.
/// - For projects, this is the unveil cost.
/// - For spells, artifacts, and weapons this is the casting cost.
/// - Schemes do not have a mana cost
pub fn mana_cost(game: &GameState, card_id: CardId) -> Option<ManaValue> {
    dispatch::perform_query(
        game,
        ManaCostQuery(card_id),
        crate::get(game.card(card_id).variant).cost.mana,
    )
}

/// Returns the mana cost for a given ability, if any. Includes the cost of the
/// card itself if it is currently face-down.
pub fn ability_mana_cost(game: &GameState, ability_id: AbilityId) -> Option<ManaValue> {
    let mut cost = if let AbilityType::Activated { cost, .. } =
        &crate::get(game.card(ability_id.card_id).variant).ability(ability_id.index).ability_type
    {
        cost.mana
    } else {
        None
    };

    if game.card(ability_id.card_id).is_face_down() {
        cost = match (cost, mana_cost(game, ability_id.card_id)) {
            (None, None) => None,
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (Some(x), Some(y)) => Some(x + y),
        };
    }

    dispatch::perform_query(game, AbilityManaCostQuery(ability_id), cost)
}

/// Returns the action point cost for a given card
pub fn action_cost(game: &GameState, card_id: CardId) -> ActionCount {
    let mut actions = crate::get(game.card(card_id).variant).cost.actions;
    if let Some(GamePrompt::PlayCardBrowser(browser)) =
        game.player(card_id.side).prompt_queue.get(0)
    {
        if browser.cards.contains(&card_id) {
            // Cards played from play browser implicitly cost 1 action point fewer
            actions = actions.saturating_sub(1);
        }
    }
    dispatch::perform_query(game, ActionCostQuery(card_id), actions)
}

/// Returns the attack power value for a given card, or 0 by default.
pub fn base_attack(game: &GameState, card_id: CardId) -> AttackValue {
    dispatch::perform_query(
        game,
        BaseAttackQuery(card_id),
        stats(game, card_id).base_attack.unwrap_or(0),
    )
}

/// Returns the health value for a given card, or 0 by default.
pub fn health(game: &GameState, card_id: CardId) -> HealthValue {
    dispatch::perform_query(
        game,
        HealthValueQuery(card_id),
        stats(game, card_id).health.unwrap_or(0),
    )
}

/// Returns the shield value for a given minion card, or 0 by default.
///
/// A `weapon_id` should be provided to determine the shield value when opposing
/// a specific weapon card.
pub fn shield(game: &GameState, minion_id: CardId, weapon_id: Option<CardId>) -> ShieldValue {
    dispatch::perform_query(
        game,
        ShieldValueQuery(ShieldCardInfo { minion_id, weapon_id }),
        stats(game, minion_id).shield.unwrap_or(0),
    )
}

/// Returns the breach value for a given card, or 0 by default.
pub fn breach(game: &GameState, card_id: CardId) -> BreachValue {
    dispatch::perform_query(
        game,
        BreachValueQuery(card_id),
        stats(game, card_id).breach.unwrap_or(0),
    )
}

/// Returns the raze cost (cost to destroy/discard when accessed) for a given
/// card, or 0 by default.
pub fn raze_cost(game: &GameState, card_id: CardId) -> RazeCost {
    dispatch::perform_query(
        game,
        RazeCostQuery(card_id),
        stats(game, card_id).raze_cost.unwrap_or(0),
    )
}

/// Look up the number of action points a player receives at the start of their
/// turn
pub fn start_of_turn_action_count(game: &GameState, side: Side) -> ActionCount {
    let default = match side {
        Side::Overlord => game_constants::OVERLORD_START_OF_TURN_ACTIONS,
        Side::Champion => game_constants::CHAMPION_START_OF_TURN_ACTIONS,
    };

    dispatch::perform_query(game, StartOfTurnActionsQuery(side), default)
}

/// Look up the number of cards the Champion player can access from the Vault
/// during the current raid
pub fn vault_access_count(game: &GameState) -> Result<u32> {
    let raid_id = game.raid()?.raid_id;
    Ok(dispatch::perform_query(game, VaultAccessCountQuery(raid_id), 1))
}

/// Look up the number of cards the Champion player can access from the Sanctum
/// during the current raid
pub fn sanctum_access_count(game: &GameState) -> Result<u32> {
    let raid_id = game.raid()?.raid_id;
    Ok(dispatch::perform_query(game, SanctumAccessCountQuery(raid_id), 1))
}

/// Looks up what type of target a given card requires
pub fn card_target_kind(game: &GameState, card_id: CardId) -> CardTargetKind {
    let definition = game.card(card_id).definition();
    if let Some(targeting) = &definition.config.custom_targeting {
        return match targeting {
            TargetRequirement::None => CardTargetKind::None,
            TargetRequirement::TargetRoom(_) => CardTargetKind::Room,
        };
    }

    match definition.card_type {
        CardType::Minion | CardType::Project | CardType::Scheme => CardTargetKind::Room,
        _ => CardTargetKind::None,
    }
}

/// Returns the highest mana cost card among those in the provided
/// `card_iterator` (breaking ties based on sorting key), or None if there is no
/// such card.
pub fn highest_cost<'a>(card_iterator: impl Iterator<Item = &'a CardState>) -> Option<CardId> {
    let cards = card_iterator.collect::<Vec<_>>();
    let max = cards.iter().filter_map(|c| crate::get(c.variant).cost.mana).max();
    let mut filtered =
        cards.into_iter().filter(|c| crate::get(c.variant).cost.mana == max).collect::<Vec<_>>();
    filtered.sort();
    filtered.first().map(|c| c.id)
}

/// Queries the maximum hand size for a player.
pub fn maximum_hand_size(game: &GameState, side: Side) -> u32 {
    dispatch::perform_query(
        game,
        MaximumHandSizeQuery(side),
        game_constants::STARTING_MAXIMUM_HAND_SIZE,
    )
}

/// Locates a minion in play, returning its current room and index position
/// within that room, if any.
pub fn minion_position(game: &GameState, minion_id: CardId) -> Option<(RoomId, usize)> {
    match game.card(minion_id).position() {
        CardPosition::Room(room_id, location) if location == RoomLocation::Defender => {
            let index = game.defender_list(room_id).iter().position(|cid| *cid == minion_id);
            index.map(|i| (room_id, i))
        }
        _ => None,
    }
}

/// Returns the position to which a card should be moved after being played by
/// the [Side] player with a given [CardTarget]. Returns `None` if no position
/// exists for this target.
pub fn played_position(
    game: &GameState,
    side: Side,
    card_id: CardId,
    target: CardTarget,
) -> Option<CardPosition> {
    Some(match game.card(card_id).definition().card_type {
        CardType::ChampionSpell | CardType::OverlordSpell => CardPosition::DiscardPile(side),
        CardType::Artifact => CardPosition::ArenaItem(ItemLocation::Artifacts),
        CardType::Ally => CardPosition::ArenaItem(ItemLocation::Evocations),
        CardType::Evocation => CardPosition::ArenaItem(ItemLocation::Evocations),
        CardType::Minion => CardPosition::Room(target.room_id().ok()?, RoomLocation::Defender),
        CardType::Project | CardType::Scheme => {
            CardPosition::Room(target.room_id().ok()?, RoomLocation::Occupant)
        }
        CardType::Riftcaller => CardPosition::Riftcaller(side),
        CardType::GameModifier => CardPosition::GameModifier,
    })
}

/// Returns a [RaidStatus] describing the high-level state of this `raid`.
pub fn raid_status(raid: &RaidData) -> RaidStatus {
    match &raid.state {
        RaidState::Step(step) => match step {
            RaidStep::Begin => RaidStatus::Begin,
            RaidStep::PopulateSummonPrompt(_)
            | RaidStep::SummonMinion(_)
            | RaidStep::DoNotSummon(_) => RaidStatus::Summon,
            RaidStep::NextEncounter
            | RaidStep::EncounterMinion(_)
            | RaidStep::PopulateEncounterPrompt(_)
            | RaidStep::UseWeapon(_)
            | RaidStep::MinionDefeated(_)
            | RaidStep::FireMinionCombatAbility(_) => RaidStatus::Encounter,
            RaidStep::PopulateApproachPrompt => RaidStatus::ApproachRoom,
            RaidStep::AccessStart
            | RaidStep::BuildAccessSet
            | RaidStep::AccessSetBuilt
            | RaidStep::RevealAccessedCards
            | RaidStep::AccessCards
            | RaidStep::PopulateAccessPrompt
            | RaidStep::StartScoringCard(_)
            | RaidStep::ChampionScoreEvent(_)
            | RaidStep::ScoreEvent(_)
            | RaidStep::ScorePointsForCard(_)
            | RaidStep::MoveToScoredPosition(_)
            | RaidStep::StartRazingCard(_, _)
            | RaidStep::RazeCard(_, _)
            | RaidStep::FinishRaid => RaidStatus::Access,
        },
        RaidState::Prompt(prompt) => prompt.status,
    }
}
