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

use data::card_name::CardName;
use data::deck::Deck;
use data::player_name::NamedPlayer;
use data::primitives::Side;
use data::user_actions::NamedDeck;
use maplit::hashmap;
use once_cell::sync::Lazy;

/// Empty Overlord deck for use in tests
pub static EMPTY_OVERLORD: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Overlord,
    identity: CardName::TestOverlordIdentity,
    cards: HashMap::new(),
});

/// Spell Overlord deck for use in tests
pub static OVERLORD_TEST_SPELLS: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Overlord,
    identity: CardName::TestOverlordIdentity,
    cards: hashmap! {CardName::TestOverlordSpell => 45},
});

/// Standard Overlord deck for use in tests
pub static CANONICAL_OVERLORD: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Overlord,
    identity: CardName::TestOverlordIdentity,
    cards: hashmap! {
        CardName::GoldMine => 3,
        CardName::ActivateReinforcements => 2,
        CardName::ResearchProject => 2,
        CardName::Gemcarver => 2,
        CardName::Coinery => 2,
        CardName::SpikeTrap => 2,
        CardName::OverwhelmingPower => 2,
        CardName::GatheringDark => 3,
        CardName::ForcedMarch => 2,
        CardName::TimeGolem => 1,
        CardName::TemporalStalker => 2,
        CardName::ShadowLurker => 3,
        CardName::SphinxOfWintersBreath => 2,
        CardName::BridgeTroll => 2,
        CardName::Stormcaller => 2,
        CardName::FireGoblin => 2
    },
});

/// Empty Champion deck for use in tests
pub static EMPTY_CHAMPION: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Champion,
    identity: CardName::TestChampionIdentity,
    cards: HashMap::new(),
});

/// Spell Overlord deck for use in tests
pub static CHAMPION_TEST_SPELLS: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Champion,
    identity: CardName::TestChampionIdentity,
    cards: hashmap! {CardName::TestChampionSpell => 45},
});

/// Standard Champion deck for use in tests
pub static CANONICAL_CHAMPION: Lazy<Deck> = Lazy::new(|| Deck {
    side: Side::Champion,
    identity: CardName::TestChampionIdentity,
    cards: hashmap! {
        CardName::Meditation => 2,
        CardName::CoupDeGrace => 3,
        CardName::ChargedStrike => 2,
        CardName::ArcaneRecovery => 3,
        CardName::StealthMission => 2,
        CardName::Preparation => 2,
        CardName::InvisibilityRing => 1,
        CardName::Accumulator => 1,
        CardName::MageGloves => 1,
        CardName::SkysReach => 2,
        CardName::MagicalResonator => 2,
        CardName::DarkGrimoire => 1,
        CardName::MaraudersAxe => 2,
        CardName::KeenHalberd => 2,
        CardName::EtherealBlade => 2,
        CardName::BowOfTheAlliance => 2
    },
});

/// Returns a canonical deck associated with the given [Side].
pub fn canonical_deck(side: Side) -> Deck {
    if side == Side::Champion {
        CANONICAL_CHAMPION.clone()
    } else {
        CANONICAL_OVERLORD.clone()
    }
}

/// Looks up the [Deck] for a named player.
pub fn deck_for_player(_: NamedPlayer, side: Side) -> Deck {
    // (Eventually different named players will have different decks)
    canonical_deck(side)
}

/// Returns a canonical deck associated with the given [NamedDeck].
pub fn named_deck(name: NamedDeck) -> Deck {
    match name {
        NamedDeck::EmptyChampion => EMPTY_CHAMPION.clone(),
        NamedDeck::ChampionTestSpells => CHAMPION_TEST_SPELLS.clone(),
        NamedDeck::CanonicalChampion => CANONICAL_CHAMPION.clone(),
        NamedDeck::EmptyOverlord => EMPTY_OVERLORD.clone(),
        NamedDeck::OverlordTestSpells => OVERLORD_TEST_SPELLS.clone(),
        NamedDeck::CanonicalOverlord => CANONICAL_OVERLORD.clone(),
    }
}