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

//! Defines card names

use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};

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
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct CardVariant {
    pub name: CardName,
    pub metadata: CardMetadata,
}

impl CardVariant {
    /// Base card variant with no upgrades or cosmetic modifications.
    pub const fn standard(name: CardName) -> Self {
        Self { name, metadata: CardMetadata { is_upgraded: false } }
    }

    /// Upgraded variant of a card
    pub const fn upgraded(name: CardName) -> Self {
        Self { name, metadata: CardMetadata { is_upgraded: true } }
    }

    /// Returns an integer which uniquely identifies this variant among all
    /// other variants.
    pub fn as_ident(&self) -> u64 {
        let result = self.name as u64;
        if self.metadata.is_upgraded {
            result + 1_000_000
        } else {
            result
        }
    }

    pub fn displayed_name(&self) -> String {
        self.name.displayed_name()
    }
}

impl Debug for CardVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)
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
    /// Spell card which is the only member of the `TestSingletonSpellSet` card
    /// set.
    TestSingletonSetSpell,
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
    TestMinionShield2Astral,
    /// Equivalent to `TestMinionEndRaid` with 3 shield point & mortal
    /// resonance
    TestMinionShield3Mortal,
    /// Minion with 5 health, 1 mana cost, and a "deal 1 damage" ability.
    TestMinionDealDamage,
    /// Minion with MINION_HEALTH health, MINION_COST mana cost, and a "the
    /// Riftcaller loses 1 mana" combat ability.
    TestMinionLoseMana,
    /// Minion with MINION_HEALTH health, MINION_COST mana cost, and a "the
    /// Riftcaller loses 1 action points" combat ability.
    TestMinionLoseActionPoints,
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
    /// Riftcaller spell with a mana cost of 0
    Test0CostSpell,
    /// Covenant ritual with a mana cost of 0
    Test0CostRitual,
    /// Riftcaller spell with a mana cost of 1
    Test1CostSpell,
    /// Riftcaller spell with a mana cost of 5
    Test5CostSpell,
    TestMinionDealDamageEndRaid,
    TestAttackWeapon,
    TestSacrificeDrawCardArtifact,
    TestWeaponReduceCostOnSuccessfulRaid,
    TestRitualGiveCurse,
    TestEvocation,
    TestAlly,
    TestRitualDeal1Damage,
    TestRitualDeal5Damage,
    /// Ritual to return all discarded cards to the Covenant's hand
    TestRitualReturnDiscardToHand,
    /// Ritual to return all 'occupant' cards to the Covenant's hand
    TestRitualReturnAllOccupantsToHand,
    /// Summon all minion cards currently in play
    TestRitualSummonAllMinions,
    /// Spell to return all Riftcaller permanents to their hand
    TestSpellReturnAllYourPermanentsToHand,
    /// Destroy all Riftcaller permanents
    TestRitualDestroyAllEnemyPermanents,
    TestAllyAccessAdditionalSanctumCard,
    TestAllyAccessAdditionalVaultCard,
    TestChargeArtifact,

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
    CovenantEmptyModifier,

    // Tutorial Effects
    TutorialDisableDrawAction,
    TutorialDisableGainMana,
    TutorialDisableRaidSanctum,
    TutorialDisableRaidVault,
    TutorialDisableRaidCrypt,
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

    // Riftcaller: Beryl
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
    NimbusEnclave,
    IlleaTheHighSage,
    MagistratesThronehall,
    IncarnationOfJustice,
    SplinterOfTwilight,
    EchoingCacophony,
    Solidarity,
    Lightbond,
    EnforcersOfSilence,
    KeepersOfTheEye,
    LiminalTransposition,
    SentinelSphinx,
    StraziharTheAllSeeing,
    GodmirSparkOfDefiance,
    OleusTheWatcher,
    TheStarseers,
    AMomentsPeace,
    BlueWarden,
    VortexPortal,
    RadiantIntervention,
    BrilliantGambit,
    EllisarForgekeeper,
    Foresee,
    LivingStone,
    SeldannaRegalPyromancer,
    LightcallersCommand,
    RolantTheRestorer,
    SpearOfUltimatum,
    RitualOfBinding,
    RiversEye,
    TheConjurersCircle,
    EriaTheGhostOfVasilor,
    LawholdCavalier,
    SealedNecropolis,
    HasteResonator,
    MaulOfDevastation,
    AngelOfUnity,
    UsilynaMasterArtificer,
    PotentialityStorm,
    AmarasDecree,
    TheHonorbound,
    DusksAscension,
    NobleMartyr,
    SariandiPhaseWalker,
    RiftAdept,
    AeonSwimmer,
    Lawbringer,
    Vengeance,
    Mazeshaper,
    EchoingValor,
    Summermorn,
    SoldierServitor,
    HauntingMelody,
    ForetellFate,
    CondemnToEternity,
    PhalanxGuardian,
    Windmare,
    TheGrandDesign,
    HealingPool,
    UsriaYinrelSpellseeker,
    PhasewarpPortal,
    DeliriumEngine,
}

impl CardName {
    /// Returns the user-visible name for this card
    pub fn displayed_name(&self) -> String {
        let custom = match self {
            Self::MaraudersAxe => "Marauder's Axe",
            Self::SphinxOfWintersBreath => "Sphinx of Winter's Breath",
            Self::WarriorsSign => "Warrior's Sign",
            Self::IlleaTheHighSage => "Illea, the High Sage",
            Self::MagistratesThronehall => "Magistrate's Thronehall",
            Self::KeepersOfTheEye => "Keepers of the Eye",
            Self::StraziharTheAllSeeing => "Strazihar, the All-Seeing",
            Self::GodmirSparkOfDefiance => "Godmir, Spark of Defiance",
            Self::OleusTheWatcher => "Oleus, the Watcher",
            Self::AMomentsPeace => "A Moment's Peace",
            Self::EllisarForgekeeper => "Ellisar, Forgekeeper",
            Self::SeldannaRegalPyromancer => "Seldanna, Regal Pyromancer",
            Self::LightcallersCommand => "Lightcaller's Command",
            Self::RolantTheRestorer => "Rolant the Restorer",
            Self::RiversEye => "River's Eye",
            Self::TheConjurersCircle => "The Conjurer's Circle",
            Self::EriaTheGhostOfVasilor => "Eria, the Ghost of Vasilor",
            Self::UsilynaMasterArtificer => "Usilyna, Master Artificer",
            Self::AmarasDecree => "Amara's Decree",
            Self::DusksAscension => "Dusk's Ascension",
            Self::SariandiPhaseWalker => "Sariandi, Phase Walker",
            Self::UsriaYinrelSpellseeker => "Usria Yinrel, Spellseeker",
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
