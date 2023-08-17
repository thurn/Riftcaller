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
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

use adapters::ServerCardId;
use game_data::card_name::CardName;
use game_data::player_name::PlayerId;
use game_data::primitives::{AttackValue, CardId, GameId, HealthValue, Lineage, ManaValue};
use protos::spelldawn::CardIdentifier;
use ulid::Ulid;

pub static NEXT_ID: AtomicU64 = AtomicU64::new(1_000_000);

pub fn generate_ids() -> (GameId, PlayerId, PlayerId) {
    let next_id = NEXT_ID.fetch_add(2, Ordering::SeqCst);
    (
        GameId::new_from_u128(next_id as u128),
        PlayerId::Database(Ulid(next_id as u128)),
        PlayerId::Database(Ulid(next_id as u128 + 1)),
    )
}

/// Looks up the test minion to use for a given lineage type.
pub fn minion_for_lineage(lineage: Lineage) -> CardName {
    match lineage {
        Lineage::Mortal => CardName::TestMortalMinion,
        Lineage::Abyssal => CardName::TestAbyssalMinion,
        Lineage::Infernal => CardName::TestInfernalMinion,
        _ => panic!("Unsupported"),
    }
}

/// Numbers which determine the boost requirement for a weapon
pub struct WeaponStats {
    pub cost: ManaValue,
    pub attack: AttackValue,
    pub boost_cost: ManaValue,
    pub boost: AttackValue,
}

/// Returns the basic mana cost to play a card with the stats in [WeaponStats]
/// and defeat a minion with `minion_health` health
pub fn cost_to_play_and_defeat(stats: WeaponStats, minion_health: HealthValue) -> ManaValue {
    (((minion_health - stats.attack) / stats.boost) * stats.boost_cost) + stats.cost
}

/// Asserts that the display names of the provided vector of [CardName]s are
/// precisely identical to the provided vector of strings.
pub fn assert_identical(expected: Vec<CardName>, actual: Vec<String>) {
    let set = expected.iter().map(CardName::displayed_name).collect::<Vec<_>>();
    assert_eq!(set, actual);
}

/// Asserts two vectors contain the same elements in any order
pub fn assert_contents_equal<T: Eq + Hash + Debug>(left: Vec<T>, right: Vec<T>) {
    let left_count = left.len();
    let right_count = right.len();
    let left_set: HashSet<T> = left.into_iter().collect();
    let right_set: HashSet<T> = right.into_iter().collect();
    assert_eq!(left_set.len(), left_count);
    assert_eq!(right_set.len(), right_count);
    assert_eq!(left_set, right_set);
}

/// Asserts that a [Result] is not an error
pub fn assert_ok<T: Debug, E: Debug>(result: &Result<T, E>) {
    assert!(result.is_ok(), "Unexpected error, got {result:?}")
}

/// Asserts that a [Result] is an error
pub fn assert_error<T: Debug, E: Debug>(result: Result<T, E>) {
    assert!(result.is_err(), "Expected an error, got {result:?}")
}

/// Creates a [CardIdentifier] representing the ability with the provided
/// `index` of this `card_id`.
pub fn ability_id(card_id: CardIdentifier, ability: u32) -> CardIdentifier {
    CardIdentifier { ability_id: Some(ability), ..card_id }
}

/// Converts a [CardIdentifier] into a [CardId].
pub fn server_card_id(card_id: CardIdentifier) -> CardId {
    match adapters::server_card_id(card_id) {
        Ok(ServerCardId::CardId(id)) => id,
        _ => panic!("Expected server card id"),
    }
}
