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

//! Fundamental types and data structures for Spelldawn

#![allow(clippy::copy_iterator)] // Suppress IntoEnumIterator warning

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use anyhow::Result;
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use ulid::Ulid;

use with_error::fail;

pub type TurnNumber = u32;
pub type ActionCount = u32;
pub type ManaValue = u32;
pub type PointsValue = u32;
pub type HealthValue = u32;
pub type AttackValue = u32;
pub type ShieldValue = u32;
pub type BreachValue = u32;
pub type ProgressValue = u32;
pub type PowerChargeValue = u32;
pub type DamageAmount = u32;
pub type RazeCost = u32;
pub type CurseCount = u32;
pub type WoundCount = u32;
pub type LeylineCount = u32;

/// Identifies one of a player's decks
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum DeckId {
    /// The current deck being used in adventure mode
    Adventure,
}

/// Identifies an ongoing game
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GameId(Ulid);

impl GameId {
    pub fn generate() -> Self {
        Self(Ulid::new())
    }

    pub fn new(ulid: Ulid) -> Self {
        Self(ulid)
    }

    pub fn new_from_u128(value: u128) -> Self {
        Self(Ulid(value))
    }

    pub fn as_u128(self) -> u128 {
        self.0 .0
    }
}

impl fmt::Debug for GameId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

/// Identifies an ongoing adventure
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AdventureId(Ulid);

impl AdventureId {
    pub fn generate() -> Self {
        Self(Ulid::new())
    }

    pub fn new(ulid: Ulid) -> Self {
        Self(ulid)
    }

    pub fn new_from_u128(value: u128) -> Self {
        Self(Ulid(value))
    }

    pub fn as_u128(self) -> u128 {
        self.0 .0
    }
}

impl fmt::Debug for AdventureId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl fmt::Display for AdventureId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

/// The two players in a game: Overlord & Champion
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd, Sequence)]
pub enum Side {
    Overlord,
    Champion,
}

impl Side {
    /// Gets the opponent of the provided player
    pub fn opponent(&self) -> Self {
        match self {
            Side::Champion => Self::Overlord,
            Side::Overlord => Self::Champion,
        }
    }

    pub fn letter(&self) -> &'static str {
        match self {
            Side::Overlord => "O",
            Side::Champion => "C",
        }
    }

    pub fn from_letter(s: impl Into<String>) -> Result<Side> {
        let letter = s.into();
        match () {
            _ if letter == "O" => Ok(Side::Overlord),
            _ if letter == "C" => Ok(Side::Champion),
            _ => fail!("Invalid side identifier"),
        }
    }
}

impl fmt::Debug for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Side::Overlord => "Overlord",
                Side::Champion => "Champion",
            }
        )
    }
}

/// Identifies a struct that is 1:1 associated with a given [Side].
pub trait HasSide {
    fn side(&self) -> Side;
}

impl HasSide for Side {
    fn side(&self) -> Side {
        *self
    }
}

/// Identifies a card in an ongoing game
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub struct CardId {
    pub side: Side,
    pub index: usize,
}

impl CardId {
    pub fn new(side: Side, index: usize) -> Self {
        Self { side, index }
    }
}

/// Identifies a struct that is 1:1 associated with a given [CardId].
pub trait HasCardId {
    fn card_id(&self) -> CardId;
}

impl HasCardId for CardId {
    fn card_id(&self) -> CardId {
        // I know this is the same as Into, I just find it less annoying to have
        // explicit types :)
        *self
    }
}

impl HasSide for CardId {
    fn side(&self) -> Side {
        self.side
    }
}

impl fmt::Debug for CardId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.side.letter(), self.index)
    }
}

/// Identifies an ability position within a card's 'abilities' vector
#[derive(PartialEq, Eq, Hash, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AbilityIndex(pub usize);

impl AbilityIndex {
    pub fn value(self) -> usize {
        self.0
    }
}

impl fmt::Debug for AbilityIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifies a struct that is 1:1 associated with a given [AbilityId].
pub trait HasAbilityId {
    fn ability_id(&self) -> AbilityId;
}

impl<T: HasAbilityId> HasCardId for T {
    fn card_id(&self) -> CardId {
        self.ability_id().card_id
    }
}

/// Identifies an ability within a card. Abilities are the only game entity
/// which may contain delegates..
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct AbilityId {
    pub card_id: CardId,
    pub index: AbilityIndex,
}

impl fmt::Debug for AbilityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.card_id.side.letter(), self.card_id.index, self.index.0)
    }
}

impl FromStr for AbilityId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let vec = s.split('/').collect::<Vec<_>>();
        if vec.len() == 3 {
            Ok(AbilityId {
                card_id: CardId::new(Side::from_letter(vec[0])?, vec[1].parse::<usize>()?),
                index: AbilityIndex(vec[2].parse::<usize>()?),
            })
        } else {
            fail!("Expected exactly two '/' characters")
        }
    }
}

impl Display for AbilityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl AbilityId {
    pub fn new(card_id: CardId, index: usize) -> Self {
        Self { card_id, index: AbilityIndex(index) }
    }

    pub fn side(&self) -> Side {
        self.card_id.side
    }
}

impl HasAbilityId for AbilityId {
    fn ability_id(&self) -> AbilityId {
        *self
    }
}

/// Represents an entity in the game which can be independently animated.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum GameObjectId {
    CardId(CardId),
    AbilityId(AbilityId),
    Deck(Side),
    DiscardPile(Side),
    Character(Side),
}

impl From<CardId> for GameObjectId {
    fn from(card_id: CardId) -> Self {
        GameObjectId::CardId(card_id)
    }
}

impl From<AbilityId> for GameObjectId {
    fn from(ability_id: AbilityId) -> Self {
        GameObjectId::AbilityId(ability_id)
    }
}

/// Uniquely identifies a raid within a given game
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct RaidId(pub u32);

impl fmt::Debug for RaidId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Uniquely identifies a minion encounter within a given game
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct MinionEncounterId(pub u32);

impl fmt::Debug for MinionEncounterId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Uniquely identifies a room access within a given game
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct RoomAccessId(pub u32);

impl fmt::Debug for RoomAccessId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Uniquely identifies a room access within a given game
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub struct BanishEventId(pub u32);

impl fmt::Debug for BanishEventId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Contains the URL of an image asset within a game
#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub address: String,
}

impl Sprite {
    pub fn new(address: impl Into<String>) -> Self {
        Self { address: address.into() }
    }
}

/// The schools of magic, which provide restrictions on players during
/// deckbuilding
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub enum School {
    Law,
    Shadow,
    Primal,
    Pact,
    Beyond,
    Neutral,
}

impl School {
    pub fn displayed_name(&self) -> &'static str {
        match self {
            School::Law => "Law",
            School::Shadow => "Shadow",
            School::Primal => "Primal",
            School::Pact => "Pact",
            School::Beyond => "Beyond",
            School::Neutral => "Neutral",
        }
    }
}

/// The five standard schools of magic, not including [School::Neutral].
pub static STANDARD_SCHOOLS: &'static [School] =
    &[School::Law, School::Shadow, School::Primal, School::Pact, School::Beyond];

/// The possible Rooms in which the Overlord player may play their cards.
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Sequence, Ord, PartialOrd,
)]
pub enum RoomId {
    /// The Overlord's deck
    Vault,
    /// The Overlord's hand
    Sanctum,
    /// The Overlord's discard pile
    Crypts,
    RoomA,
    RoomB,
    RoomC,
    RoomD,
    RoomE,
}

pub static INNER_ROOMS: &'static [RoomId] = &[RoomId::Vault, RoomId::Sanctum, RoomId::Crypts];

impl RoomId {
    /// An 'inner room' is one of the three predefined rooms for the Overlord
    /// player's deck, hand, and discard pile. Inner rooms cannot contain
    /// Schemes or Projects.
    pub fn is_inner_room(&self) -> bool {
        matches!(self, RoomId::Vault | RoomId::Sanctum | RoomId::Crypts)
    }

    /// An 'outer room' is any room other than the three pre-defined inner rooms
    pub fn is_outer_room(&self) -> bool {
        !self.is_inner_room()
    }
}

/// Identifies a struct that is 1:1 associated with a given [RoomId].
pub trait HasRoomId {
    fn room_id(&self) -> RoomId;
}

/// Identifies a struct which identifies a [RoomId] at compile time.
pub trait RoomIdMarker {
    fn room_id() -> RoomId;
}

pub struct RoomIdVault;
impl RoomIdMarker for RoomIdVault {
    fn room_id() -> RoomId {
        RoomId::Vault
    }
}

pub struct RoomIdSanctum;
impl RoomIdMarker for RoomIdSanctum {
    fn room_id() -> RoomId {
        RoomId::Sanctum
    }
}

pub struct RoomIdCrypts;
impl RoomIdMarker for RoomIdCrypts {
    fn room_id() -> RoomId {
        RoomId::Crypts
    }
}

/// Identifies a struct that is 1:1 associated with a given [RaidId].
pub trait HasRaidId {
    fn raid_id(&self) -> RaidId;
}

/// Used to control where a card is rendered within a room
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub enum RoomLocation {
    Defender,
    Occupant,
}

/// Used to control where an item is rendered within the Champion's item display
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub enum ItemLocation {
    Artifacts,
    Evocations,
    Allies,
}

/// Rarity of a card, used to determine how likely it is to appear in randomized
/// rewards.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Riftcaller,

    /// Card cannot be obtained via random rewards
    None,
}

/// Possible types of cards
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub enum CardType {
    Riftcaller,
    GameModifier,
    Spell,
    Artifact,
    Evocation,
    Ally,
    Scheme,
    Ritual,
    Project,
    Minion,
}

impl CardType {
    pub fn is_spell(&self) -> bool {
        matches!(self, CardType::Spell | CardType::Ritual)
    }

    /// Returns the english article 'a' or 'an' appropriate for this card type.
    pub fn article(&self) -> &'static str {
        match self {
            Self::Spell
            | Self::Ritual
            | Self::Minion
            | Self::Project
            | Self::Scheme
            | Self::Riftcaller
            | Self::GameModifier => "a",
            Self::Artifact | Self::Evocation | Self::Ally => "an",
        }
    }
}

impl Display for CardType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CardType::Spell => write!(f, "Spell"),
            CardType::Artifact => write!(f, "Artifact"),
            CardType::Evocation => write!(f, "Evocation"),
            CardType::Ally => write!(f, "Ally"),
            CardType::Ritual => write!(f, "Spell"),
            CardType::Minion => write!(f, "Minion"),
            CardType::Project => write!(f, "Project"),
            CardType::Scheme => write!(f, "Scheme"),
            CardType::Riftcaller => write!(f, "Riftcaller"),
            CardType::GameModifier => write!(f, "Modifier"),
        }
    }
}

/// Subtypes of cards
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Display, EnumString, Serialize, Deserialize)]
pub enum CardSubtype {
    /// Cards with the "Trap" subtype cannot be summoned
    Trap,
    /// Cards with the "Duskbound" subtype can be summoned at the end of the
    /// Champion's turn.
    Duskbound,
    /// Cards with the "Nightbound" subtype can be summoned during the
    /// Overlord's turn.
    Nightbound,
    /// Cards with the "Roombound" subtype can be summoned when a room is
    /// approached by the Champion.
    Roombound,
    /// Cards with the "Summonbound" subtype can be summoned when a face-down
    /// minion is approached.
    Summonbound,
    /// Cards with the "Enchanted" subtype can have their power charges removed
    /// by the Overlord.
    Enchanted,

    Weapon,
    Silvered,
    Conjuration,
    Raid,
    Runic,
    Charge,
    Warrior,
    Cleric,
    Noble,
    Mystic,
    Mage,
    Expedition,
    Augury,
}

/// An interval of time in milliseconds
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Milliseconds(pub u32);

/// Contextual information to include with client response payloads for use in
/// logging and crash attribution.
#[derive(Copy, Clone, Debug)]
pub enum ResponseContext {
    Default,
    Adventure(AdventureId),
    Game(GameId),

    // Explicitly clear stored metadata when leaving a game/adventure
    LeaveAdventure,
    LeaveGame,
}

/// Identifies whether some game update was caused by a player taking an
/// explicit game action such as the 'initiate raid' action, or by a card
/// effect.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum InitiatedBy {
    GameAction,
    Ability(AbilityId),
}

impl InitiatedBy {
    pub fn is_game_action(&self) -> bool {
        matches!(self, InitiatedBy::GameAction)
    }

    pub fn is_ability(&self) -> bool {
        matches!(self, InitiatedBy::Ability(_))
    }

    pub fn ability_id(&self) -> Option<AbilityId> {
        match self {
            InitiatedBy::GameAction => None,
            InitiatedBy::Ability(id) => Some(*id),
        }
    }

    pub fn card_id(&self) -> Option<CardId> {
        self.ability_id().map(|a| a.card_id)
    }
}