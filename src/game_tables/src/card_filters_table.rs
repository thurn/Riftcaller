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

use std::collections::HashMap;
use std::sync::Mutex;

use adventure_data::card_filter_data::{CardFilter, UpgradedStatus};
use anyhow::Result;
use core_data::adventure_primitives::CardFilterId;
use core_data::game_primitives::{CardSubtype, CardType, Rarity, Side};
use enumset::{EnumSet, EnumSetType};
use once_cell::sync::Lazy;

use crate::csv_datatypes::CardFilterRow;

static ROWS: Lazy<Mutex<Vec<CardFilterRow>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn clear() {
    ROWS.lock().expect("Unable to lock ROWS").clear();
}

pub fn import_row(row: CardFilterRow) {
    ROWS.lock().expect("Unable to lock ROWS").push(row)
}

pub fn build() -> Result<HashMap<CardFilterId, CardFilter>> {
    let rows = ROWS.lock().expect("Unable to lock ROWS");
    Ok(rows.iter().map(|row| (CardFilterId::new(row.id), build_filter(row))).collect())
}

fn build_filter(row: &CardFilterRow) -> CardFilter {
    let mut result = CardFilter::new(row.category_operator);
    insert(&mut result.upgraded, row.upgraded_no, UpgradedStatus::Default);
    insert(&mut result.upgraded, row.upgraded_yes, UpgradedStatus::Upgraded);
    insert(&mut result.rarity, row.rarity_common, Rarity::Common);
    insert(&mut result.rarity, row.rarity_uncommon, Rarity::Uncommon);
    insert(&mut result.rarity, row.rarity_rare, Rarity::Rare);
    insert(&mut result.rarity, row.rarity_basic, Rarity::Basic);
    insert(&mut result.rarity, row.rarity_identity, Rarity::Identity);
    insert(&mut result.rarity, row.rarity_none, Rarity::None);
    insert(&mut result.card_types, row.type_riftcaller, CardType::Riftcaller);
    insert(&mut result.card_types, row.type_chapter, CardType::Chapter);
    insert(&mut result.card_types, row.type_game_modifier, CardType::GameModifier);
    insert(&mut result.card_types, row.type_sigil, CardType::Sigil);
    insert(&mut result.card_types, row.type_scheme, CardType::Scheme);
    insert(&mut result.card_types, row.type_spell, CardType::Spell);
    insert(&mut result.card_types, row.type_ritual, CardType::Ritual);
    insert(&mut result.card_types, row.type_evocation, CardType::Evocation);
    insert(&mut result.card_types, row.type_ally, CardType::Ally);
    insert(&mut result.card_types, row.type_project, CardType::Project);
    insert(&mut result.card_types, row.type_artifact, CardType::Artifact);
    insert(&mut result.card_types, row.type_minion, CardType::Minion);
    insert(&mut result.sides, row.side_covenant, Side::Covenant);
    insert(&mut result.sides, row.side_riftcaller, Side::Riftcaller);
    insert(&mut result.card_subtypes, row.subtype_weapon, CardSubtype::Weapon);
    result
}

fn insert<T: EnumSetType>(set: &mut EnumSet<T>, value: bool, flag: T) {
    if value {
        set.insert(flag);
    }
}
