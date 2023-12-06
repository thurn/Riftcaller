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

use std::collections::{HashMap, HashSet};

use adventure_data::adventure::{AdventureConfiguration, AdventureState};
use core_data::adventure_primitives::{Coins, TilePosition};
use core_data::game_primitives::{AdventureId, Side};
use game_data::card_name::{CardName, CardVariant};
use game_data::deck::Deck;
use game_data::player_name::PlayerId;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;

use crate::test_session::TestSession;
use crate::test_session_builder::TestSessionBuilder;

pub struct TestAdventure {
    side: Side,
    coins: Coins,
    visiting_position: Option<TilePosition>,
    deck: HashMap<CardVariant, u32>,
    collection: HashMap<CardVariant, u32>,
}

impl TestAdventure {
    pub fn new(side: Side) -> Self {
        Self {
            side,
            coins: Coins(999),
            visiting_position: None,
            deck: HashMap::new(),
            collection: HashMap::new(),
        }
    }

    pub fn coins(mut self, coins: Coins) -> Self {
        self.coins = coins;
        self
    }

    pub fn visiting_position(mut self, position: TilePosition) -> Self {
        self.visiting_position = Some(position);
        self
    }

    pub fn deck_card(mut self, card: CardName, quantity: u32) -> Self {
        self.deck.insert(CardVariant::standard(card), quantity);
        self
    }

    pub fn collection_card(mut self, card: CardName, quantity: u32) -> Self {
        self.collection.insert(CardVariant::standard(card), quantity);
        self
    }

    /// Creates a new adventure session using the configuration provided.
    pub fn build(self) -> TestSession {
        TestSessionBuilder::new().adventure(self).build()
    }

    pub fn build_adventure_state_internal(self, player_id: PlayerId) -> AdventureState {
        let id = AdventureId::generate();
        let mut revealed_regions = HashSet::new();
        revealed_regions.insert(1);
        let deck = Deck { side: self.side, schools: vec![], identities: vec![], cards: self.deck };
        let config = AdventureConfiguration {
            player_id,
            side: self.side,
            rng: Some(Xoshiro256StarStar::seed_from_u64(314159265358979323)),
        };

        AdventureState {
            id,
            side: self.side,
            coins: self.coins,
            visiting_position: self.visiting_position,
            outcome: None,
            tiles: HashMap::new(),
            revealed_regions,
            deck,
            collection: self.collection,
            config,
        }
    }
}
