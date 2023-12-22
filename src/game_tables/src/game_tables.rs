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
use serde::de::DeserializeOwned;

use crate::csv_datatypes::{CardFilterRow, NarrativeEventDetailsRow, NarrativeEventRow};

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

static GAME_TABLES: Lazy<GameTables> =
    Lazy::new(|| initialize().expect("Error building GameTables"));

const NARRATIVE_EVENTS: &[u8] = include_bytes!("narrative_events.csv");
const NARRATIVE_EVENT_DETAILS: &[u8] = include_bytes!("narrative_event_details.csv");
const CARD_FILTERS: &[u8] = include_bytes!("card_filters.csv");

struct GameTables {
    card_filters: HashMap<CardFilterId, CardFilter>,
    narrative_events: HashMap<NarrativeEventId, NarrativeEventData>,
}

/// Parse the imported CSVs file and return the result as a [GameTables] struct.
fn initialize() -> Result<GameTables> {
    let card_filters = deserialize::<CardFilterRow>(CARD_FILTERS)?;
    let narrative_events = deserialize::<NarrativeEventRow>(NARRATIVE_EVENTS)?;
    let narrative_event_details = deserialize::<NarrativeEventDetailsRow>(NARRATIVE_EVENT_DETAILS)?;

    Ok(GameTables {
        card_filters: card_filters_table::build(card_filters)?,
        narrative_events: narrative_events_table::build(narrative_events, narrative_event_details)?,
    })
}

fn deserialize<T: DeserializeOwned>(content: &'static [u8]) -> Result<Vec<T>> {
    let mut result = vec![];
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_reader(content);
    for row in reader.deserialize() {
        let value: csv::Result<T> = row;
        result.push(value?)
    }
    Ok(result)
}
