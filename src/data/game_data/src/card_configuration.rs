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

//! Data structures for defining card rules -- the parts of a card which do not
//! vary from game to game.

#![allow(clippy::use_self)] // Required to use EnumKind

use std::fmt;
use std::fmt::{Debug, Formatter};

use anyhow::Result;
use core_data::adventure_primitives::{Coins, Skill};
use core_data::game_primitives;
use core_data::game_primitives::{
    ActionCount, AttackValue, BreachValue, CardId, HealthValue, ManaValue, PointsValue,
    PowerChargeValue, ProgressValue, RazeCost, Resonance, RoomId, School, ShieldValue, Sprite,
};
use enum_kinds::EnumKind;
use enumset::EnumSet;

use crate::card_name::CardMetadata;
use crate::game_state::GameState;
use crate::special_effects::{ProjectileData, TimedEffectData};
use crate::text::TextElement;

/// A cost represented by custom functions.
///
/// For cards that enter face-up, this cost is expected to be played
/// immediately. Otherwise, the cost is paid at the time of reveal. Custom costs
/// are not automatically reflected in rules text, so the implementor should add
/// them manually. Constraints on how a card or ability can be played (such as
/// "activate only once per turn" or "play only if you control a mortal minion")
/// are also represented as costs.
#[derive(Clone)]
pub struct CustomCost<T> {
    /// Whether this cost can currently be paid
    pub can_pay: fn(&GameState, T) -> bool,
    /// Mutate the game to pay this cost. Should fail if `can_pay` would return
    /// false.
    pub pay: fn(&mut GameState, T) -> Result<()>,

    /// Optionally, a description of this cost to include before the ':'
    /// character.
    pub description: Option<TextElement>,
}

impl<T> Debug for CustomCost<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "CustomCost")
    }
}

/// Cost to play or summon a card, or activate an ability
#[derive(Debug, Clone)]
pub struct Cost<T> {
    /// Cost in mana
    pub mana: Option<ManaValue>,
    /// Cost in action points
    pub actions: ActionCount,
    /// A custom cost or requirement to play this card/activate this ability.
    /// See [CustomCost].
    pub custom_cost: Option<CustomCost<T>>,
}

impl<T> Cost<T> {
    pub fn zero() -> Self {
        Self { mana: None, actions: 0, custom_cost: None }
    }

    pub fn add_mana_cost(mut self, cost: ManaValue) -> Self {
        self.mana = Some(self.mana.unwrap_or_default() + cost);
        self
    }
}

impl<T> Default for Cost<T> {
    fn default() -> Self {
        Self { mana: None, actions: 1, custom_cost: None }
    }
}

/// Possible additional costs for using a weapon
#[derive(Debug, Clone)]
pub enum CustomWeaponCost {
    ActionPoints(ActionCount),
}

/// Possible alternate costs for activating an attack boost.
#[derive(Debug, Clone)]
pub enum CustomBoostCost {
    PowerCharges(PowerChargeValue),
}

/// An activated ability used by Weapons to increase their attack value by
/// paying a mana cost during a raid encounter. Can be used any number of times.
#[derive(Debug, Clone, Default)]
pub struct AttackBoost {
    /// Mana cost to activate an instance of this boost
    pub cost: ManaValue,
    /// Additional cost to use this weapon
    pub custom_weapon_cost: Option<CustomWeaponCost>,
    /// Additional custom cost to pay to activate an instance of this boost
    pub custom_boost_cost: Option<CustomBoostCost>,
    /// Bonus to attack added for each activation
    pub bonus: AttackValue,
}

impl AttackBoost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mana_cost(mut self, mana: ManaValue) -> Self {
        self.cost = mana;
        self
    }

    pub fn bonus(mut self, bonus: AttackValue) -> Self {
        self.bonus = bonus;
        self
    }

    pub fn custom_weapon_cost(mut self, custom_weapon_cost: CustomWeaponCost) -> Self {
        self.custom_weapon_cost = Some(custom_weapon_cost);
        self
    }

    pub fn custom_boost_cost(mut self, custom_cost: CustomBoostCost) -> Self {
        self.custom_boost_cost = Some(custom_cost);
        self
    }
}

/// Scoring information about a card
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct SchemePoints {
    /// Required number of progress counters to score this card
    pub progress_requirement: ProgressValue,
    /// Number of points received for scoring this card
    pub points: PointsValue,
}

/// Base card state values
#[derive(Debug, Clone, Default)]
pub struct CardStats {
    /// Damage required to destroy this card
    pub health: Option<HealthValue>,
    /// Mana cost required in order to interact with this card
    pub shield: Option<ShieldValue>,
    /// Allows a weapon to bypass some amount of shield points.
    pub breach: Option<BreachValue>,
    /// Cost that must be paid to discard/destroy this card when accessed.
    pub raze_cost: Option<RazeCost>,
    /// Base damage dealt by this card during an encounter
    pub base_attack: Option<AttackValue>,
    /// An increase in base attack damage for a fixed cost which an ability can
    /// apply to this card
    pub attack_boost: Option<AttackBoost>,
    /// Level Requirement & points for scoring this card
    pub scheme_points: Option<SchemePoints>,
}

pub type RoomPredicate<T> = fn(&GameState, T, RoomId) -> bool;

/// Allows cards and abilities to provide special targeting behavior.
#[derive(Clone, EnumKind)]
#[enum_kind(TargetRequirementKind)]
pub enum TargetRequirement<T> {
    /// No target required
    None,
    /// Target a specific room when played. Only rooms for which the provided
    /// [RoomPredicate] returns true are considered valid targets. This
    /// predicate is expected to check room identity based on the card's text,
    /// it does not need to verify that e.g. the card can currently be played.
    TargetRoom(RoomPredicate<T>),
}

impl<T: Copy> TargetRequirement<T> {
    /// Returns true if there are currently any valid targets for this
    /// TargetRequirement in the current game state.
    pub fn has_valid_targets(&self, game: &GameState, id: T) -> bool {
        match self {
            TargetRequirement::None => true,
            TargetRequirement::TargetRoom(predicate) => {
                game_primitives::ROOMS.iter().any(|room_id| predicate(game, id, *room_id))
            }
        }
    }
}

impl<T> Debug for TargetRequirement<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let kind: TargetRequirementKind = self.into();
        write!(f, "{kind:?}")
    }
}

/// Configuration for an Identity card
#[derive(Debug)]
pub struct IdentityConfig {
    pub starting_coins: Coins,
    pub secondary_schools: Vec<School>,
    pub skills: Vec<Skill>,
    pub bio: &'static str,
}

/// Individual card configuration; properties which are not universal for all
/// cards
#[derive(Debug, Default)]
pub struct CardConfig {
    /// Basic numerical properties of this card.
    pub stats: CardStats,
    /// Optionally, the resonance for this card. Weapon cards can only interact
    /// with Minion cards that have a matching resonance.
    pub resonance: EnumSet<Resonance>,
    /// Targeting requirements for this card, e.g. to target a room.
    pub custom_targeting: Option<TargetRequirement<CardId>>,
    /// A projectile to use when this card's combat ability triggers
    pub combat_projectile: Option<ProjectileData>,
    /// Alternate image to display to identify players in the arena
    pub player_portrait: Option<Sprite>,
    /// Content to display behind the main image
    pub image_background: Option<Sprite>,
    /// A visual effect associated with this card, selected via a prompt choice.
    /// Used to e.g. indicate targeting.
    pub visual_effect: Option<TimedEffectData>,
    /// Optionally, a clarifying note about how this card functions.
    pub note: Option<String>,
    /// Configuration for a Riftcaller or Chapter card
    pub identity: Option<IdentityConfig>,
    /// Which card variant does this definition correspond to?
    ///
    /// It is never necessary to specify this value when building a card
    /// definition, we automatically write the correct variant to each
    /// definition after construction.
    pub metadata: CardMetadata,
}

#[derive(Debug, Default)]
pub struct CardConfigBuilder {
    config: CardConfig,
}

impl CardConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> CardConfig {
        self.config
    }

    pub fn health(mut self, value: HealthValue) -> Self {
        self.config.stats.health = Some(value);
        self
    }

    pub fn shield(mut self, value: ShieldValue) -> Self {
        self.config.stats.shield = Some(value);
        self
    }

    pub fn breach(mut self, value: BreachValue) -> Self {
        self.config.stats.breach = Some(value);
        self
    }

    pub fn raze_cost(mut self, value: RazeCost) -> Self {
        self.config.stats.raze_cost = Some(value);
        self
    }

    pub fn base_attack(mut self, value: AttackValue) -> Self {
        self.config.stats.base_attack = Some(value);
        self
    }

    pub fn attack_boost(mut self, value: AttackBoost) -> Self {
        self.config.stats.attack_boost = Some(value);
        self
    }

    pub fn scheme_points(mut self, value: SchemePoints) -> Self {
        self.config.stats.scheme_points = Some(value);
        self
    }

    pub fn resonance(mut self, resonance: Resonance) -> Self {
        self.config.resonance.insert(resonance);
        self
    }

    pub fn custom_targeting(mut self, targeting: TargetRequirement<CardId>) -> Self {
        self.config.custom_targeting = Some(targeting);
        self
    }

    pub fn combat_projectile(mut self, projectile: ProjectileData) -> Self {
        self.config.combat_projectile = Some(projectile);
        self
    }

    pub fn visual_effect(mut self, effect: TimedEffectData) -> Self {
        self.config.visual_effect = Some(effect);
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.config.note = Some(note.into());
        self
    }

    pub fn identity(mut self, config: IdentityConfig) -> Self {
        self.config.identity = Some(config);
        self
    }
}
