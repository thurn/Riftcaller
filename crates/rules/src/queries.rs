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

use crate::dispatch;
use data::card_definition::CardStats;
use data::card_state::CardPosition;
use data::delegates;
use data::game::GameState;
use data::primitives::{
    ActionCount, AttackValue, BoostCount, CardId, HealthValue, ManaValue, ShieldValue, Side,
};

/// Returns the top card of the indicated player's deck, selecting randomly if no cards are known
/// to be present there. Returns None if the deck is empty.
pub fn top_of_deck(game: &GameState, side: Side) -> Option<CardId> {
    game.cards(side)
        .iter()
        .filter(|c| c.position == CardPosition::DeckTop(side))
        .max_by_key(|c| c.sorting_key)
        .map_or_else(|| game.random_card(CardPosition::DeckUnknown(side)), |card| Some(card.id))
}

/// Obtain the [CardStats] for a given card
pub fn stats(game: &GameState, card_id: impl Into<CardId>) -> &CardStats {
    &crate::get(game.card(card_id).name).config.stats
}

pub fn mana_cost(game: &GameState, card_id: impl Into<CardId> + Copy) -> Option<ManaValue> {
    dispatch::perform_query(
        game,
        delegates::get_mana_cost,
        card_id.into(),
        crate::get(game.card(card_id).name).cost.mana,
    )
}

pub fn action_cost(game: &GameState, card_id: impl Into<CardId> + Copy) -> ActionCount {
    dispatch::perform_query(
        game,
        delegates::get_action_cost,
        card_id.into(),
        crate::get(game.card(card_id).name).cost.actions,
    )
}

pub fn attack(game: &GameState, card_id: impl Into<CardId> + Copy) -> AttackValue {
    dispatch::perform_query(
        game,
        delegates::get_attack_value,
        card_id.into(),
        stats(game, card_id).base_attack.unwrap_or(0),
    )
}

pub fn health(game: &GameState, card_id: impl Into<CardId> + Copy) -> HealthValue {
    dispatch::perform_query(
        game,
        delegates::get_health_value,
        card_id.into(),
        stats(game, card_id).health.unwrap_or(0),
    )
}

pub fn shield(game: &GameState, card_id: impl Into<CardId> + Copy) -> ShieldValue {
    dispatch::perform_query(
        game,
        delegates::get_shield_value,
        card_id.into(),
        stats(game, card_id).shield.unwrap_or(0),
    )
}

pub fn boost_count(game: &GameState, card_id: impl Into<CardId> + Copy) -> BoostCount {
    dispatch::perform_query(
        game,
        delegates::get_boost_count,
        card_id.into(),
        game.card(card_id).data.boost_count,
    )
}