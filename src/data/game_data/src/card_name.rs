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

//! Defines card names

use std::cmp::Ordering;

use convert_case::{Case, Casing};
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::text::TextElement;

/// Describes function & cosmetic differences for a card with a given name.
#[derive(
    PartialEq, Eq, Hash, Default, Debug, Copy, Clone, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct CardMetadata {
    pub is_upgraded: bool,
    pub full_art: bool,
}

impl CardMetadata {
    /// Returns one of two values based on whether the card is upgraded
    pub fn upgrade<T>(&self, normal: T, upgraded: T) -> T {
        if self.is_upgraded {
            upgraded
        } else {
            normal
        }
    }

    pub fn upgraded_only_text(&self, text: Vec<TextElement>) -> Vec<TextElement> {
        if self.is_upgraded {
            text
        } else {
            Vec::new()
        }
    }
}

/// Identifies a specific card version within cards with the same name, covering
/// both cosmetic and functional distinctions. Cards with the same variant are
/// visually and functionally identical under the rules of the game.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct CardVariant {
    pub name: CardName,
    pub metadata: CardMetadata,
}

impl CardVariant {
    /// Base card variant with no upgrades or cosmetic modifications.
    pub const fn standard(name: CardName) -> Self {
        Self { name, metadata: CardMetadata { is_upgraded: false, full_art: false } }
    }

    /// Upgraded variant of a card
    pub const fn upgraded(name: CardName) -> Self {
        Self { name, metadata: CardMetadata { is_upgraded: true, full_art: false } }
    }

    /// Returns an integer which uniquely identifies this variant among all
    /// other variants.
    pub fn as_ident(&self) -> u64 {
        let result = self.name as u64;
        match (self.metadata.is_upgraded, self.metadata.full_art) {
            (true, true) => result + 3_000_000,
            (true, false) => result + 2_000_000,
            (false, true) => result + 1_000_000,
            (false, false) => result,
        }
    }

    pub fn displayed_name(&self) -> String {
        self.name.displayed_name()
    }
}

impl From<CardVariant> for CardName {
    fn from(value: CardVariant) -> Self {
        value.name
    }
}

/// Possible names of cards.
///
/// This enum is used to connect the state of a card to its game rules.
#[derive(
    PartialEq, Eq, Hash, Debug, Copy, Clone, Display, EnumString, Serialize, Deserialize, Sequence,
)]
pub enum CardName {
    // When renaming a card, add  #[serde(alias = "OldName")] to preserve
    // serialization

    // Cards for use in tests
    TestSpell,
    TestRitual,
    /// Scheme requiring 3 progress to score 10 points
    TestScheme3_10,
    /// Scheme requiring 4 progress to score 20 points
    TestScheme4_20,
    /// Scheme requiring 1 progress to score 10 points
    TestScheme1_10,
    /// Blank project with a mana cost of 2
    TestProject2Cost3Raze,
    /// Minion with 5 health, 3 mana cost, and an "end the raid" ability.
    TestMinionEndRaid,
    /// Equivalent to `TestMinionEndRaid` with 1 shield point.
    TestMinionShield1Infernal,
    /// Equivalent to `TestMinionEndRaid` with 2 shield point & abyssal
    /// resonance
    TestMinionShield2Abyssal,
    /// Minion with 5 health, 1 mana cost, and a "deal 1 damage" ability.
    TestMinionDealDamage,
    /// Minion with MINION_HEALTH health, MINION_COST mana cost, and a "the
    /// Champion loses 1 mana" ability.
    TestMinionLoseMana,
    /// Minion with the 'infernal' resonance, MINION_HEALTH health, and an 'end
    /// raid' ability.
    TestInfernalMinion,
    /// Minion with the 'astral' resonance, MINION_HEALTH health, and an 'end
    /// raid' ability.
    TestAstralMinion,
    /// Minion with the 'mortal' resonance, MINION_HEALTH health, and an 'end
    /// raid' ability.
    TestMortalMinion,
    /// Minion with the 'astral' resonance, MINION_HEALTH health, and an 'end
    /// raid' ability, and 1 shield point
    TestAstralMinion1Shield,
    /// Minion with 'mortal' resonance, an 'end the raid' ability, and 2 health
    TestMortalMinion2Health,
    /// Weapon with 2 attack and no boost.
    TestWeapon2Attack,
    /// Weapon with 2 attack and a '1 mana: +2 attack' boost.
    TestWeapon2Attack12Boost,
    /// Weapon with 3 attack and a '1 mana: +2 attack' boost.
    TestWeapon3Attack12Boost3Cost,
    /// Weapon with 4 attack and a '1 mana: +2 attack' boost.
    TestWeapon4Attack12Boost,
    /// Weapon with 5 attack and no boost
    TestWeapon5AttackInfernal,
    /// Astral weapon with 3 attack and a '1 mana: +2 attack' boost.
    TestAstralWeapon,
    /// Infernal weapon with 3 attack and a '1 mana: +2 attack' boost.
    TestInfernalWeapon,
    /// Mortal weapon with 3 attack and a '1 mana: +2 attack' boost.
    TestMortalWeapon,
    /// Artifact which stores mana on play, with the activated ability to take
    /// mana from it
    TestActivatedAbilityTakeMana,
    /// Duskbound project which stores mana on summon, with a triggered ability
    /// to take mana at dusk.
    TestProjectTriggeredAbilityTakeManaAtDusk,
    /// Roombound project which can be summoned & sacrificed to end the current
    /// raid.
    TestProjectSacrificeToEndRaid,
    /// Project with two subtypes
    TestDuskAndNightboundProject,
    /// Project with the "Duskbound" subtype
    TestDuskboundProject,
    /// Project with the "Nightbound" subtype
    TestNightboundProject,
    /// Project with the "Trap" subtype
    TestTrapProject,
    /// Project with the "Summonbound" subtype
    TestSummonboundProject,
    /// Project with the "Roombound" subtype
    TestRoomboundProject,
    /// Champion spell with a mana cost of 0
    Test0CostSpell,
    /// Champion spell with a mana cost of 1
    Test1CostSpell,
    TestMinionDealDamageEndRaid,
    TestAttackWeapon,
    TestSacrificeDrawCardArtifact,
    TestWeaponReduceCostOnSuccessfulRaid,
    TestSpellGiveCurse,
    TestEvocation,
    TestSpellDeal1Damage,
    TestSpellDeal5Damage,
    /// Ritual to return all discarded cards to the Overlord's hand
    TestRitualReturnDiscardToHand,
    /// Ritual to return all 'occupant' cards to the Overlord's hand
    TestRitualReturnAllOccupantsToHand,

    // Proof of Concept
    GoldMine,
    Meditation,
    CoupDeGrace,
    ChargedStrike,
    StealthMission,
    Preparation,
    InvisibilityRing,
    Accumulator,
    MageGloves,
    MagicalResonator,
    DarkGrimoire,
    MaraudersAxe,
    KeenHalberd,
    BowOfTheAlliance,
    ActivateReinforcements,
    ResearchProject,
    Gemcarver,
    SpikeTrap,
    OverwhelmingPower,
    ForcedMarch,
    TimeGolem,
    ShadowLurker,
    SphinxOfWintersBreath,
    BridgeTroll,
    Stormcaller,
    EnneraImrisBloodBound,
    ArisFeyTheRadiantSun,
    TelantesDugothEarthbreaker,
    AndvariEstNightsWarden,
    UbrasEfarisTimeShaper,

    // Modifier card which has no effect
    OverlordEmptyModifier,

    // Tutorial Effects
    TutorialDisableDrawAction,
    TutorialDisableGainMana,
    TutorialDisableRaidSanctum,
    TutorialDisableRaidVault,
    TutorialDisableRaidCrypts,
    TutorialDisableRaidOuter,
    TutorialDisableRaidContinue,
    TutorialDisableEndRaid,
    TutorialForceSanctumScore,

    // Basic
    ArcaneRecovery,
    EldritchSurge,
    Lodestone,
    ManaBattery,
    Contemplate,
    AncestralKnowledge,
    SimpleBlade,
    SimpleAxe,
    SimpleBow,
    SimpleClub,
    SimpleHammer,
    SimpleSpear,
    SimpleStaff,
    EtherealBlade,
    Conspire,
    Devise,
    Machinate,
    GatheringDark,
    Coinery,
    Leyline,
    OreRefinery,
    Crab,
    FireGoblin,
    Toucan,
    Frog,
    Scout,
    Captain,

    // Spelldawn: Beryl
    Restoration,
    StrikeTheHeart,
    EnduringRadiance,
    SiftTheSands,
    HolyAura,
    Pathfinder,
    StaffOfTheValiant,
    Triumph,
    SpearOfConquest,
    BladeOfReckoning,
    Resolution,
    StarlightLantern,
    WarriorsSign,
    EmpyrealChorus,
    StarfieldOmen,
    Visitation,
    AstrianOracle,
    ResplendentChanneler,
    StalwartProtector,
    Dawnwarden,
    ChainsOfMortality,
    PhaseDoor,
    SpellcraftRitualist,
    BackupPlan,
    Voidstep,
    Skyprism,
    Keensight,
    ShieldOfTheFlames,
    EtherealIncursion,
    TimeStop,
    EquivalentExchange,
    Foebane,
    EtherealForm,
    PlanarSanctuary,
    ChainsOfBinding,
    WhipOfDisjunction,
    Glimmersong,
    DelveIntoDarkness,
    KnowledgeOfTheBeyond,
    ZainCunningDiplomat,
    IlleasTheHighSage,
    MagistratesThronehall,
    IncarnationOfJustice,
    SplinterOfTwilight,
    EchoingCacophony,
    Solidarity,
    Lightbond,
}

impl CardName {
    /// Returns the user-visible name for this card
    pub fn displayed_name(&self) -> String {
        let custom = match self {
            Self::MaraudersAxe => "Marauder's Axe",
            Self::SphinxOfWintersBreath => "Sphinx of Winter's Breath",
            Self::WarriorsSign => "Warrior's Sign",
            Self::ZainCunningDiplomat => "Zain, Cunning Diplomat",
            Self::IlleasTheHighSage => "Illeas, The High Sage",
            Self::MagistratesThronehall => "Magistrate's Thronehall",
            _ => "",
        };

        if custom.is_empty() {
            format!("{self}").from_case(Case::Pascal).to_case(Case::Title)
        } else {
            custom.to_string()
        }
    }

    /// Returns true if this card is a test blank
    pub fn is_test_card(&self) -> bool {
        self.displayed_name().starts_with("Test")
    }
}

impl PartialOrd<Self> for CardName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CardName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}
