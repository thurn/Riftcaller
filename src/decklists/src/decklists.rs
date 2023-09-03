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

//! Contains preconfigured card lists

use std::collections::HashMap;

use game_data::card_name::{CardName, CardVariant};
use game_data::deck::Deck;
use game_data::primitives::{School, Side};
use maplit::hashmap;
use once_cell::sync::Lazy;
use user_action_data::NamedDeck;

/// Empty Overlord deck for use in tests
pub static EMPTY_OVERLORD: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Overlord,
    schools: vec![],
    riftcallers: vec![],
    cards: HashMap::new(),
});

/// Spell Overlord deck for use in tests
pub static OVERLORD_TEST_SPELLS: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Overlord,
    schools: vec![],
    riftcallers: vec![],
    cards: hashmap! {CardVariant::standard(CardName::TestOverlordSpell) => 45},
});

/// Basic Overlord starter deck in adventure mode
pub static BASIC_OVERLORD: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Overlord,
    schools: vec![],
    riftcallers: vec![],
    cards: hashmap! {
        CardVariant::standard(CardName::Conspire) => 3,
        CardVariant::standard(CardName::Devise) => 3,
        CardVariant::standard(CardName::Machinate) => 3,
        CardVariant::standard(CardName::GatheringDark) => 3,
        CardVariant::standard(CardName::Coinery) => 3,
        CardVariant::standard(CardName::Leyline) => 3,
        CardVariant::standard(CardName::OreRefinery) => 3,
        CardVariant::standard(CardName::Crab) => 3,
        CardVariant::standard(CardName::FireGoblin) => 3,
        CardVariant::standard(CardName::Toucan) => 3,
        CardVariant::standard(CardName::Frog) => 3,
        CardVariant::standard(CardName::Scout) => 3,
        CardVariant::standard(CardName::Captain) => 3
    },
});

/// Basic Champion starter deck in adventure mode
pub static TUTORIAL_OVERLORD: Lazy<Deck> =
    Lazy::new(|| Deck { riftcallers: vec![], ..BASIC_OVERLORD.clone() });

/// Standard Overlord deck for use in benchmarks
pub static CANONICAL_OVERLORD: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Overlord,
    schools: vec![School::Law],
    riftcallers: vec![],
    cards: hashmap! {
        CardVariant::standard(CardName::GoldMine) => 3,
        CardVariant::standard(CardName::ActivateReinforcements) => 2,
        CardVariant::standard(CardName::ResearchProject) => 2,
        CardVariant::standard(CardName::Gemcarver) => 2,
        CardVariant::standard(CardName::Coinery) => 2,
        CardVariant::standard(CardName::SpikeTrap) => 2,
        CardVariant::standard(CardName::OverwhelmingPower) => 2,
        CardVariant::standard(CardName::GatheringDark) => 3,
        CardVariant::standard(CardName::ForcedMarch) => 2,
        CardVariant::standard(CardName::TimeGolem) => 1,
        CardVariant::standard(CardName::TemporalStalker) => 2,
        CardVariant::standard(CardName::ShadowLurker) => 3,
        CardVariant::standard(CardName::SphinxOfWintersBreath) => 2,
        CardVariant::standard(CardName::BridgeTroll) => 2,
        CardVariant::standard(CardName::Stormcaller) => 2,
        CardVariant::standard(CardName::FireGoblin) => 2
    },
});

/// Empty Champion deck for use in tests
pub static EMPTY_CHAMPION: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Champion,
    schools: vec![],
    riftcallers: vec![],
    cards: HashMap::new(),
});

/// Spell Overlord deck for use in tests
pub static CHAMPION_TEST_SPELLS: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Champion,
    schools: vec![],
    riftcallers: vec![],
    cards: hashmap! {CardVariant::standard(CardName::TestChampionSpell) => 45},
});

/// Basic Champion starter deck in adventure mode
pub static BASIC_CHAMPION: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Champion,
    schools: vec![],
    riftcallers: vec![],
    cards: hashmap! {
        CardVariant::standard(CardName::ArcaneRecovery) => 3,
        CardVariant::standard(CardName::EldritchSurge) => 3,
        CardVariant::standard(CardName::Lodestone) => 3,
        CardVariant::standard(CardName::ManaBattery) => 3,
        CardVariant::standard(CardName::Contemplate) => 3,
        CardVariant::standard(CardName::AncestralKnowledge) => 3,
        CardVariant::standard(CardName::SimpleBlade) => 3,
        CardVariant::standard(CardName::SimpleAxe) => 3,
        CardVariant::standard(CardName::SimpleBow) => 3,
        CardVariant::standard(CardName::SimpleClub) => 3,
        CardVariant::standard(CardName::SimpleHammer) => 3,
        CardVariant::standard(CardName::SimpleSpear) => 3,
        CardVariant::standard(CardName::EtherealBlade) => 3,
    },
});

/// Basic Champion starter deck in adventure mode
pub static TUTORIAL_CHAMPION: Lazy<Deck> =
    Lazy::new(|| Deck { riftcallers: vec![], ..BASIC_CHAMPION.clone() });

/// Standard Champion deck for use in benchmarks
pub static CANONICAL_CHAMPION: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Champion,
    schools: vec![School::Primal],
    riftcallers: vec![],
    cards: hashmap! {
        CardVariant::standard(CardName::Meditation) => 2,
        CardVariant::standard(CardName::CoupDeGrace) => 3,
        CardVariant::standard(CardName::ChargedStrike) => 2,
        CardVariant::standard(CardName::ArcaneRecovery) => 3,
        CardVariant::standard(CardName::StealthMission) => 2,
        CardVariant::standard(CardName::Preparation) => 2,
        CardVariant::standard(CardName::InvisibilityRing) => 1,
        CardVariant::standard(CardName::Accumulator) => 1,
        CardVariant::standard(CardName::MageGloves) => 1,
        CardVariant::standard(CardName::ManaBattery) => 2,
        CardVariant::standard(CardName::MagicalResonator) => 2,
        CardVariant::standard(CardName::DarkGrimoire) => 1,
        CardVariant::standard(CardName::MaraudersAxe) => 2,
        CardVariant::standard(CardName::KeenHalberd) => 2,
        CardVariant::standard(CardName::EtherealBlade) => 2,
        CardVariant::standard(CardName::BowOfTheAlliance) => 2,
    },
});

/// Returns the basic deck associated with the given [Side].
pub fn basic_deck(side: Side) -> Deck {
    if side == Side::Champion {
        BASIC_CHAMPION.clone()
    } else {
        BASIC_OVERLORD.clone()
    }
}

/// Returns a canonical deck associated with the given [Side].
pub fn canonical_deck(side: Side) -> Deck {
    if side == Side::Champion {
        CANONICAL_CHAMPION.clone()
    } else {
        CANONICAL_OVERLORD.clone()
    }
}

/// Returns a canonical deck associated with the given [NamedDeck].
pub fn named_deck(name: NamedDeck) -> Deck {
    match name {
        NamedDeck::EmptyChampion => EMPTY_CHAMPION.clone(),
        NamedDeck::ChampionTestSpells => CHAMPION_TEST_SPELLS.clone(),
        NamedDeck::BasicChampion => BASIC_CHAMPION.clone(),
        NamedDeck::TutorialChampion => TUTORIAL_CHAMPION.clone(),
        NamedDeck::CanonicalChampion => CANONICAL_CHAMPION.clone(),
        NamedDeck::EmptyOverlord => EMPTY_OVERLORD.clone(),
        NamedDeck::OverlordTestSpells => OVERLORD_TEST_SPELLS.clone(),
        NamedDeck::CanonicalOverlord => CANONICAL_OVERLORD.clone(),
        NamedDeck::BasicOverlord => BASIC_OVERLORD.clone(),
        NamedDeck::TutorialOverlord => TUTORIAL_OVERLORD.clone(),
    }
}
