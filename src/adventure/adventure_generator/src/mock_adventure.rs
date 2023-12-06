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

//! Generates world maps for the 'adventure' game mode

use std::collections::{HashMap, HashSet};

use adventure_data::adventure::{AdventureConfiguration, AdventureState, TileEntity, TileState};
use core_data::adventure_primitives::{RegionId, TilePosition};
use core_data::game_primitives::AdventureId;
use game_data::card_name::CardVariant;
use game_data::deck::Deck;

const TOP_LEFT: u8 = 0b00100000;
const TOP_RIGHT: u8 = 0b00010000;
const RIGHT: u8 = 0b00001000;
const BOTTOM_RIGHT: u8 = 0b00000100;
const BOTTOM_LEFT: u8 = 0b00000010;
const LEFT: u8 = 0b00000001;

pub const BATTLE_POSITION: TilePosition = TilePosition { x: 3, y: 2 };

/// Builds an 'adventure' mode world map for use in tests
pub fn create(
    id: AdventureId,
    config: AdventureConfiguration,
    deck: Deck,
    collection: HashMap<CardVariant, u32>,
    identities: Option<TileEntity>,
    draft: Option<TileEntity>,
    shop: Option<TileEntity>,
    battle: Option<TileEntity>,
) -> AdventureState {
    let mut tiles = HashMap::new();

    add_tile(&mut tiles, -3, 2, "hexGrassySandPalms02");
    add_tile(&mut tiles, -2, 2, "hexGrassySandPalms03");
    add_tile(&mut tiles, -1, 2, "hexPlainsCold03");
    add_tile(&mut tiles, 0, 2, "hexMarsh00");
    add_tile(&mut tiles, 1, 2, "hexPlainsHalflingVillage00");
    add_tile(&mut tiles, 2, 2, "hexDirtInn00");

    if let Some(b) = battle {
        add_with_entity_and_road(
            &mut tiles,
            3,
            2,
            "hexPlains00",
            b,
            road(TOP_RIGHT | BOTTOM_LEFT, 0),
        );
    } else {
        add_with_road(
            &mut tiles,
            BATTLE_POSITION.x,
            BATTLE_POSITION.y,
            "hexPlains00",
            road(TOP_RIGHT | BOTTOM_LEFT, 0),
        );
    }

    add_tile(&mut tiles, 4, 2, "hexPlainsSmithy00");
    add_tile(&mut tiles, -4, 1, "hexGrassySandPalms01");
    add_tile(&mut tiles, -3, 1, "hexPlainsFarm02");
    add_tile(&mut tiles, -2, 1, "hexWoodlands02");
    add_tile(&mut tiles, -1, 1, "hexHills02");
    add_tile(&mut tiles, 0, 1, "hexHills00");
    add_tile(&mut tiles, 1, 1, "hexHighlands02");
    add_with_road(&mut tiles, 2, 1, "hexScrublands01", road(TOP_RIGHT | BOTTOM_LEFT, 0));
    add_tile(&mut tiles, 3, 1, "hexPlainsFarm01");
    add_tile(&mut tiles, -3, 0, "hexDirtCastle00");
    add_with_road(&mut tiles, -2, 0, "hexPlains00", road(RIGHT | BOTTOM_LEFT, 0));
    add_with_road(&mut tiles, -1, 0, "hexPlains02", road(RIGHT | LEFT, 1));

    if let Some(s) = identities {
        add_with_entity_and_road(&mut tiles, 0, 0, "hexScrublands01", s, road(RIGHT | LEFT, 0));
    } else {
        add_with_road(&mut tiles, 0, 0, "hexScrublands01", road(RIGHT | LEFT, 0));
    }

    add_with_road(&mut tiles, 1, 0, "hexPlains01", road(RIGHT | LEFT, 0));
    add_with_road(&mut tiles, 2, 0, "hexPlains02", road(BOTTOM_RIGHT | LEFT | TOP_RIGHT, 0));
    add_tile(&mut tiles, 3, 0, "hexWoodlands00");
    add_tile(&mut tiles, 4, 0, "hexJungle02");
    add_tile(&mut tiles, -4, -1, "hexPlainsFarm01");
    add_with_road(&mut tiles, -3, -1, "hexScrublands01", road(TOP_RIGHT | BOTTOM_LEFT, 1));
    add_tile(&mut tiles, -2, -1, "hexTropicalPlains00");
    add_tile(&mut tiles, -1, -1, "hexSwamp01");

    if let Some(d) = draft {
        add_with_entity(&mut tiles, 0, -1, "hexMountain03", d);
    }

    add_tile(&mut tiles, 1, -1, "hexPlainsFarm00");
    add_with_road(&mut tiles, 2, -1, "hexPlains00", road(TOP_LEFT | BOTTOM_RIGHT, 0));
    add_tile(&mut tiles, 3, -1, "hexJungle03");
    add_with_road(&mut tiles, -3, -2, "hexScrublands00", road(TOP_RIGHT | BOTTOM_LEFT, 1));

    if let Some(s) = shop {
        add_with_entity(&mut tiles, -2, -2, "hexForestBroadleafForester00", s);
    }

    add_tile(&mut tiles, -1, -2, "hexSwamp00");
    add_tile(&mut tiles, 0, -2, "hexSwamp03");
    add_tile(&mut tiles, 1, -2, "hexForestBroadleaf00");
    add_tile(&mut tiles, 2, -2, "hexHills02");
    add_with_road(&mut tiles, 3, -2, "hexPlains00", road(TOP_LEFT | BOTTOM_RIGHT, 1));
    add_tile(&mut tiles, 4, -2, "hexJungle00");

    tiles.extend(hidden_tiles(2).into_iter());

    let mut revealed_regions = HashSet::new();
    revealed_regions.insert(1);
    let side = config.side;

    AdventureState {
        id,
        side,
        visiting_position: Some(TilePosition { x: 0, y: 0 }),
        outcome: None,
        coins: crate::STARTING_COINS,
        tiles,
        revealed_regions,
        config,
        deck,
        collection,
    }
}

fn hidden_tiles(region_id: RegionId) -> HashMap<TilePosition, TileState> {
    let mut result = HashMap::new();

    add_tile(&mut result, -4, 7, "hexHillsColdSnowTransition01");
    add_with_road(&mut result, -3, 7, "hexScrublands01", road(TOP_LEFT | BOTTOM_RIGHT, 0));
    add_tile(&mut result, -2, 7, "hexMountainCave00");
    add_tile(&mut result, -1, 7, "hexMountain01");
    add_tile(&mut result, 0, 7, "hexMountain00");
    add_tile(&mut result, 1, 7, "hexForestPine03");
    add_tile(&mut result, 2, 7, "hexForestPineClearing00");
    add_tile(&mut result, 3, 7, "hexForestPineLoggingCamp00");
    add_tile(&mut result, -3, 6, "hexHighlands01");
    add_with_road(&mut result, -2, 6, "hexPlains01", road(TOP_LEFT | BOTTOM_RIGHT, 0));
    add_tile(&mut result, -1, 6, "hexPlainsTemple00");
    add_tile(&mut result, 0, 6, "hexForestPine00");
    add_tile(&mut result, 1, 6, "hexForestPine01");
    add_tile(&mut result, 2, 6, "hexForestPine02");
    add_tile(&mut result, 3, 6, "hexPlainsWalledCity00");
    add_tile(&mut result, 4, 6, "hexHills00");
    add_tile(&mut result, -4, 5, "hexDesertYellowMesasCave00");
    add_tile(&mut result, -3, 5, "hexPlainsFarm02");
    add_with_road(&mut result, -2, 5, "hexPlains02", road(TOP_LEFT | BOTTOM_LEFT | RIGHT, 0));
    add_with_road(&mut result, -1, 5, "hexScrublands03", road(LEFT | RIGHT, 0));
    add_with_road(&mut result, 0, 5, "hexPlains00", road(LEFT | RIGHT, 1));
    add_with_road(&mut result, 1, 5, "hexPlains01", road(LEFT | RIGHT, 0));
    add_with_road(&mut result, 2, 5, "hexScrublands02", road(LEFT | BOTTOM_RIGHT, 0));
    add_tile(&mut result, 3, 5, "hexHighlands03");
    add_with_road(&mut result, -3, 4, "hexGrassySand01", road(LEFT | RIGHT, 0));
    add_with_road(&mut result, -2, 4, "hexDirt01", road(LEFT | TOP_RIGHT, 0));
    add_tile(&mut result, -1, 4, "hexPlainsFarm01");
    add_tile(&mut result, 0, 4, "hexPlainsVillage03");
    add_tile(&mut result, 1, 4, "hexPlainsFarm00");
    add_tile(&mut result, 2, 4, "hexBog02");
    add_with_road(&mut result, 3, 4, "hexPlains01", road(TOP_LEFT | BOTTOM_RIGHT, 0));
    add_tile(&mut result, 4, 4, "hexHillsCold00");
    add_tile(&mut result, -4, 3, "hexDesertYellowHills03");
    add_tile(&mut result, -3, 3, "hexDesertYellowDirtDunes03");
    add_tile(&mut result, -2, 3, "hexDesertYellowCactiForest02");
    add_tile(&mut result, -1, 3, "hexForestBurnedDirt02");
    add_tile(&mut result, 0, 3, "hexGrassySandPalms03");
    add_tile(&mut result, 1, 3, "hexDesertYellowSaltFlat00");
    add_tile(&mut result, 2, 3, "hexHighlands00");
    add_with_road(&mut result, 3, 3, "hexPlains00", road(TOP_LEFT | BOTTOM_LEFT, 0));

    for (_, state) in result.iter_mut() {
        state.region_id = region_id;
    }

    result
}

fn road(edges: u8, variant: u8) -> String {
    format!("hexRoad-{:06b}-0{}", edges, variant)
}

fn add_tile(map: &mut HashMap<TilePosition, TileState>, x: i32, y: i32, sprite: &'static str) {
    map.insert(TilePosition { x, y }, TileState::with_sprite(sprite));
}

fn add_with_road(
    map: &mut HashMap<TilePosition, TileState>,
    x: i32,
    y: i32,
    sprite: &'static str,
    road: impl Into<String>,
) {
    // Road hex names are numbered clockwise from top-left
    map.insert(
        TilePosition { x, y },
        TileState { road: Some(road.into()), ..TileState::with_sprite(sprite) },
    );
}

fn add_with_entity(
    map: &mut HashMap<TilePosition, TileState>,
    x: i32,
    y: i32,
    sprite: &'static str,
    entity: TileEntity,
) {
    map.insert(
        TilePosition { x, y },
        TileState { entity: Some(entity), ..TileState::with_sprite(sprite) },
    );
}

fn add_with_entity_and_road(
    map: &mut HashMap<TilePosition, TileState>,
    x: i32,
    y: i32,
    sprite: &'static str,
    entity: TileEntity,
    road: impl Into<String>,
) {
    map.insert(
        TilePosition { x, y },
        TileState {
            entity: Some(entity),
            road: Some(road.into()),
            ..TileState::with_sprite(sprite)
        },
    );
}
