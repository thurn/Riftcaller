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

//! Contains preconfigured card lists

use std::collections::HashMap;

use core_data::game_primitives::{School, Side};
use game_data::card_name::{CardName, CardVariant};
use game_data::deck::Deck;
use maplit::hashmap;
use once_cell::sync::Lazy;
use user_action_data::NamedDeck;

/// Empty Covenant deck for use in tests
pub static EMPTY_COVENANT: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Covenant,
    schools: vec![],
    identities: vec![],
    sigils: vec![],
    cards: HashMap::new(),
});

/// Spell Covenant deck for use in tests
pub static COVENANT_TEST_SPELLS: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Covenant,
    schools: vec![],
    identities: vec![],
    sigils: vec![],
    cards: hashmap! {CardVariant::standard(CardName::TestRitual) => 45},
});

/// Basic Covenant starter deck in adventure mode
pub static BASIC_COVENANT: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Covenant,
    schools: vec![],
    identities: vec![],
    sigils: vec![],
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

/// Basic Riftcaller starter deck in adventure mode
pub static TUTORIAL_COVENANT: Lazy<Deck> =
    Lazy::new(|| Deck { identities: vec![], ..BASIC_COVENANT.clone() });

/// Standard Covenant deck for use in benchmarks
pub static CANONICAL_COVENANT: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Covenant,
    schools: vec![School::Law],
    identities: vec![],
    sigils: vec![],
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
        CardVariant::standard(CardName::ShadowLurker) => 3,
        CardVariant::standard(CardName::SphinxOfWintersBreath) => 2,
        CardVariant::standard(CardName::BridgeTroll) => 2,
        CardVariant::standard(CardName::Stormcaller) => 2,
        CardVariant::standard(CardName::FireGoblin) => 2
    },
});

/// Empty Riftcaller deck for use in tests
pub static EMPTY_RIFTCALLER: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Riftcaller,
    schools: vec![],
    identities: vec![],
    sigils: vec![],
    cards: HashMap::new(),
});

/// Spell Covenant deck for use in tests
pub static RIFTCALLER_TEST_SPELLS: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Riftcaller,
    schools: vec![],
    identities: vec![],
    sigils: vec![],
    cards: hashmap! {CardVariant::standard(CardName::TestSpell) => 45},
});

/// Basic Riftcaller starter deck in adventure mode
pub static BASIC_RIFTCALLER: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Riftcaller,
    schools: vec![],
    identities: vec![],
    sigils: vec![],
    cards: hashmap! {
        CardVariant::standard(CardName::ArcaneRecovery) => 3,
        CardVariant::standard(CardName::EldritchSurge) => 3,
        CardVariant::standard(CardName::Lodestone) => 3,
        CardVariant::standard(CardName::ManaBattery) => 3,
        CardVariant::standard(CardName::Contemplate) => 2,
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

/// Basic Riftcaller starter deck in adventure mode
pub static TUTORIAL_RIFTCALLER: Lazy<Deck> =
    Lazy::new(|| Deck { identities: vec![], ..BASIC_RIFTCALLER.clone() });

/// Standard Riftcaller deck for use in benchmarks
pub static CANONICAL_RIFTCALLER: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Riftcaller,
    schools: vec![School::Primal],
    identities: vec![],
    sigils: vec![],
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
    if side == Side::Riftcaller {
        BASIC_RIFTCALLER.clone()
    } else {
        BASIC_COVENANT.clone()
    }
}

/// Returns a canonical deck associated with the given [Side].
pub fn canonical_deck(side: Side) -> Deck {
    if side == Side::Riftcaller {
        CANONICAL_RIFTCALLER.clone()
    } else {
        CANONICAL_COVENANT.clone()
    }
}

/// Returns a canonical deck associated with the given [NamedDeck].
pub fn named_deck(name: NamedDeck) -> Deck {
    match name {
        NamedDeck::EmptyRiftcaller => EMPTY_RIFTCALLER.clone(),
        NamedDeck::RiftcallerTestSpells => RIFTCALLER_TEST_SPELLS.clone(),
        NamedDeck::BasicRiftcaller => BASIC_RIFTCALLER.clone(),
        NamedDeck::TutorialRiftcaller => TUTORIAL_RIFTCALLER.clone(),
        NamedDeck::CanonicalRiftcaller => CANONICAL_RIFTCALLER.clone(),
        NamedDeck::EmptyCovenant => EMPTY_COVENANT.clone(),
        NamedDeck::CovenantTestSpells => COVENANT_TEST_SPELLS.clone(),
        NamedDeck::CanonicalCovenant => CANONICAL_COVENANT.clone(),
        NamedDeck::BasicCovenant => BASIC_COVENANT.clone(),
        NamedDeck::TutorialCovenant => TUTORIAL_COVENANT.clone(),
    }
}
