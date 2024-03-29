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

use adventure_data::adventure::{
    AdventureConfiguration, AdventureScreens, AdventureState, TileIcon, TileState, WorldMap,
};
use adventure_data::adventure_effect_data::AdventureEffect;
use core_data::adventure_primitives::{CardFilterId, NarrativeEventId, TilePosition};
use core_data::game_primitives::{AdventureId, Side};
use game_data::card_name::{CardName, CardVariant};
use game_data::deck::Deck;

const TOP_LEFT: u8 = 0b00100000;
const TOP_RIGHT: u8 = 0b00010000;
const RIGHT: u8 = 0b00001000;
const BOTTOM_RIGHT: u8 = 0b00000100;
const BOTTOM_LEFT: u8 = 0b00000010;
const LEFT: u8 = 0b00000001;

/// Builds an 'adventure' mode world map for use in tests
pub fn create(config: AdventureConfiguration) -> AdventureState {
    let side = config.side;
    let deck = match side {
        Side::Covenant => Deck {
            identities: vec![CardVariant::standard(CardName::RiversEye)],
            ..decklists::BASIC_COVENANT.clone()
        },
        Side::Riftcaller => Deck {
            identities: vec![CardVariant::standard(CardName::OleusTheWatcher)],
            ..decklists::BASIC_RIFTCALLER.clone()
        },
    };

    let mut tiles = HashMap::new();
    add_tile(&mut tiles, -3, 2, "hexGrassySandPalms02");
    add_tile(&mut tiles, -2, 2, "hexGrassySandPalms03");
    add_tile(&mut tiles, -1, 2, "hexPlainsCold03");
    add_tile(&mut tiles, 0, 2, "hexMarsh00");
    add_tile(&mut tiles, 1, 2, "hexPlainsHalflingVillage00");
    add_tile(&mut tiles, 2, 2, "hexDirtInn00");
    add_with_road(&mut tiles, 3, 2, "hexPlains00", road(TOP_RIGHT | BOTTOM_LEFT, 0));
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
    add_with_road(&mut tiles, 0, 0, "hexScrublands01", road(RIGHT | LEFT, 0));
    add_with_road(&mut tiles, 1, 0, "hexPlains01", road(RIGHT | LEFT, 0));
    add_with_road(&mut tiles, 2, 0, "hexPlains02", road(BOTTOM_RIGHT | LEFT | TOP_RIGHT, 0));
    add_tile(&mut tiles, 3, 0, "hexWoodlands00");
    add_tile(&mut tiles, 4, 0, "hexJungle02");
    add_tile(&mut tiles, -4, -1, "hexPlainsFarm01");
    add_with_road(&mut tiles, -3, -1, "hexScrublands01", road(TOP_RIGHT | BOTTOM_LEFT, 1));
    add_tile(&mut tiles, -2, -1, "hexTropicalPlains00");
    add_tile(&mut tiles, -1, -1, "hexSwamp01");
    add_with_entity(
        &mut tiles,
        0,
        -1,
        "hexMountain03",
        AdventureEffect::Draft(CardFilterId::new(3)),
        TileIcon::Draft,
    );
    add_with_entity(
        &mut tiles,
        1,
        -1,
        "hexPlainsFarm00",
        AdventureEffect::NarrativeEvent(NarrativeEventId::new(1)),
        TileIcon::NarrativeEvent,
    );

    add_with_road(&mut tiles, 2, -1, "hexPlains00", road(TOP_LEFT | BOTTOM_RIGHT, 0));
    add_tile(&mut tiles, 3, -1, "hexJungle03");
    add_with_road(&mut tiles, -3, -2, "hexScrublands00", road(TOP_RIGHT | BOTTOM_LEFT, 1));
    add_with_entity(
        &mut tiles,
        -2,
        -2,
        "hexForestBroadleafForester00",
        AdventureEffect::Shop(CardFilterId::new(2)),
        TileIcon::Shop,
    );
    add_tile(&mut tiles, -1, -2, "hexSwamp00");
    add_tile(&mut tiles, 0, -2, "hexSwamp03");
    add_tile(&mut tiles, 1, -2, "hexForestBroadleaf00");
    add_tile(&mut tiles, 2, -2, "hexHills02");
    add_with_road(&mut tiles, 3, -2, "hexPlains00", road(TOP_LEFT | BOTTOM_RIGHT, 1));
    add_tile(&mut tiles, 4, -2, "hexJungle00");

    let mut revealed_regions = HashSet::new();
    revealed_regions.insert(1);
    let side = config.side;

    AdventureState {
        id: AdventureId::generate(),
        side,
        outcome: None,
        coins: crate::STARTING_COINS,
        world_map: WorldMap { tiles },
        screens: AdventureScreens::default(),
        config,
        deck,
    }
}

fn _hidden_tiles() -> HashMap<TilePosition, TileState> {
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
    effect: AdventureEffect,
    icon: TileIcon,
) {
    map.insert(
        TilePosition { x, y },
        TileState { on_visited: Some(effect), icons: vec![icon], ..TileState::with_sprite(sprite) },
    );
}
