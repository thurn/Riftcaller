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

//! Data structures for defining card rules -- the parts of a card which do not
//! vary from game to game.

#![allow(clippy::use_self)] // Required to use EnumKind

use std::fmt;
use std::fmt::{Debug, Formatter};

use anyhow::Result;
use enum_kinds::EnumKind;

use crate::card_name::{CardMetadata, CardName, CardVariant};
use crate::card_set_name::CardSetName;
use crate::delegate_data::Delegate;
use crate::game_state::GameState;
use crate::primitives::{
    AbilityId, AbilityIndex, ActionCount, AttackValue, BreachValue, CardId, CardSubtype, CardType,
    HealthValue, ManaValue, PointsValue, PowerChargeValue, ProgressValue, Rarity, RazeCost,
    Resonance, RoomId, School, ShieldValue, Side, Sprite,
};
use crate::special_effects::ProjectileData;
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

/// Cost to play, unveil, or summon a card, or activate an ability
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

impl<T> Default for Cost<T> {
    fn default() -> Self {
        Self { mana: None, actions: 1, custom_cost: None }
    }
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
    /// Additional custom cost to pay to activate an instance of this boost
    pub custom_cost: Option<CustomBoostCost>,
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

    pub fn custom_cost(mut self, custom_cost: CustomBoostCost) -> Self {
        self.custom_cost = Some(custom_cost);
        self
    }
}

/// Scoring information about a card
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct SchemePoints {
    /// Required number of level counters to score this card
    pub level_requirement: ProgressValue,
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

impl<T> Debug for TargetRequirement<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind: TargetRequirementKind = self.into();
        write!(f, "{kind:?}")
    }
}

/// Predicate which provides additional restrictions on whether an ability can
/// be activated. If not specified, normal activation rules apply.
pub type CanActivate = fn(&GameState, AbilityId) -> bool;

/// Possible types of ability
#[derive(Debug, Clone, EnumKind)]
#[enum_kind(AbilityTypeKind)]
pub enum AbilityType {
    /// Standard abilities function at all times without requiring activation.
    Standard,

    /// Activated abilities have an associated cost in order to be used.
    Activated {
        cost: Cost<AbilityId>,
        target_requirement: TargetRequirement<AbilityId>,
        can_activate: Option<CanActivate>,
    },

    /// Abilities which have no effect, but simply provide additional card text.
    TextOnly,
}

/// Abilities are the unit of action in Spelldawn. Their behavior is provided by
/// the Delegate system, see delegate_data for more information.
#[derive(Debug)]
pub struct Ability {
    pub ability_type: AbilityType,
    pub text: Vec<TextElement>,
    pub delegates: Vec<Delegate>,
}

/// Individual card configuration; properties which are not universal for all
/// cards
#[derive(Debug, Default)]
pub struct CardConfig {
    pub stats: CardStats,
    pub resonance: Option<Resonance>,
    pub custom_targeting: Option<TargetRequirement<CardId>>,
    /// A projectile to use when this card's combat ability triggers
    pub combat_projectile: Option<ProjectileData>,
    /// Alternate image to display to identify players in the arena
    pub player_portrait: Option<Sprite>,
    /// Content to display behind the main image
    pub image_background: Option<Sprite>,
    /// Which card variant does this definition correspond to?
    ///
    /// It is never necessary to specify this value when building a card
    /// definition, we automatically write the correct variant to each
    /// definition after construction. A value of 'None' is treated
    /// identically to the standard variant.
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
        self.config.resonance = Some(resonance);
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
}

/// The fundamental object defining the behavior of a given card in Spelldawn
///
/// This struct's top-level fields should be universal properties which need to
/// be set by every card
#[derive(Debug)]
pub struct CardDefinition {
    pub name: CardName,
    pub sets: Vec<CardSetName>,
    pub cost: Cost<CardId>,
    pub image: Sprite,
    pub card_type: CardType,
    pub subtypes: Vec<CardSubtype>,
    pub side: Side,
    pub school: School,
    pub rarity: Rarity,
    pub abilities: Vec<Ability>,
    pub config: CardConfig,
}

impl CardDefinition {
    pub fn variant(&self) -> CardVariant {
        CardVariant { name: self.name, metadata: self.config.metadata }
    }

    /// Returns the ability at the given index. Panics if no ability with this
    /// index exists.
    pub fn ability(&self, index: AbilityIndex) -> &Ability {
        &self.abilities[index.value()]
    }

    /// Iterator over all [AbilityId]s of a card.
    pub fn ability_ids(&self, card_id: CardId) -> impl Iterator<Item = AbilityId> {
        (0..self.abilities.len()).map(move |i| AbilityId::new(card_id, i))
    }

    pub fn is_spell(&self) -> bool {
        self.card_type.is_spell()
    }

    pub fn is_scheme(&self) -> bool {
        self.card_type == CardType::Scheme
    }

    pub fn is_minion(&self) -> bool {
        self.card_type == CardType::Minion
    }
}
