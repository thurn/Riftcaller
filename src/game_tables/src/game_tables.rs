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

//! Parses game-specific CSV files into data structures

mod card_filters_table;
mod csv_datatypes;
mod narrative_events_table;

use std::collections::HashMap;

use adventure_data::card_filter_data::CardFilter;
use adventure_data::narrative_event_data::NarrativeEventData;
use anyhow::Result;
use core_data::adventure_primitives::{CardFilterId, NarrativeEventId};
use csv::{ReaderBuilder, Trim};
use once_cell::sync::Lazy;

use crate::csv_datatypes::{CardFilterRow, NarrativeEventDetailsRow, NarrativeEventRow};

static GAME_TABLES: Lazy<GameTables> = Lazy::new(|| GameTables {
    card_filters: card_filters_table::build().expect("Error parsing card filters"),
    narrative_events: narrative_events_table::build().expect("Error parsing narrative events"),
});

/// Retrieves the value for a [CardFilter] with a given [CardFilterId].
///
/// Panics if no such [CardFilter] exists.
pub fn card_filter(id: CardFilterId) -> &'static CardFilter {
    GAME_TABLES.card_filters.get(&id).unwrap_or_else(|| panic!("Card filter {id:?} not found"))
}

/// Retrieves the value for a [NarrativeEventData] with a given
/// [NarrativeEventId].
///
/// Panics if no such narrative event exists.
pub fn narrative_event(id: NarrativeEventId) -> &'static NarrativeEventData {
    GAME_TABLES
        .narrative_events
        .get(&id)
        .unwrap_or_else(|| panic!("Narrative event {id:?} not found"))
}

struct GameTables {
    card_filters: HashMap<CardFilterId, CardFilter>,
    narrative_events: HashMap<NarrativeEventId, NarrativeEventData>,
}

/// Parse the provided `content` string as a CSV file and set the result as the
/// value of the game table with the provided table number.
pub fn import(table_number: i32, content: &str) -> Result<()> {
    // Please keep this list in sync with the one in riftcaller.proto!
    const NARRATIVE_EVENTS_TABLE: i32 = 2;
    const NARRATIVE_EVENT_DETAILS_TABLE: i32 = 3;
    const CARD_FILTERS_TABLE: i32 = 4;

    let mut reader = ReaderBuilder::new().trim(Trim::All).from_reader(content.as_bytes());

    match table_number {
        NARRATIVE_EVENTS_TABLE => {
            narrative_events_table::clear_rows();
            for result in reader.deserialize() {
                let row: NarrativeEventRow = result?;
                narrative_events_table::import_row(row);
            }
        }
        NARRATIVE_EVENT_DETAILS_TABLE => {
            narrative_events_table::clear_details();
            for result in reader.deserialize() {
                let row: NarrativeEventDetailsRow = result?;
                narrative_events_table::import_details_row(row);
            }
        }
        CARD_FILTERS_TABLE => {
            card_filters_table::clear();
            for result in reader.deserialize() {
                let row: CardFilterRow = result?;
                card_filters_table::import_row(row);
            }
        }
        _ => {}
    }

    Ok(())
}
