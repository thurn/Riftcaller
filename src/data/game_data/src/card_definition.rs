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
    HealthValue, ManaValue, PointsValue, PowerChargeValue, ProgressValue, Rarity, RazeCost, RoomId,
    School, ShieldValue, Side, Sprite,
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

impl<T> Debug for TargetRequirement<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    Activated { cost: Cost<AbilityId>, target_requirement: TargetRequirement<AbilityId> },
}

/// Abilities are the unit of action in Spelldawn. Their behavior is provided by
/// the Delegate system, see delegate_data for more information.
#[derive(Debug)]
pub struct Ability {
    pub ability_type: AbilityType,
    pub text: Vec<TextElement>,
    pub delegates: Vec<Delegate>,
}

impl Ability {
    pub fn new(text: Vec<TextElement>) -> Self {
        Self { ability_type: AbilityType::Standard, text, delegates: vec![] }
    }

    pub fn new_with_delegate(text: Vec<TextElement>, delegate: Delegate) -> Self {
        Self { ability_type: AbilityType::Standard, text, delegates: vec![delegate] }
    }

    pub fn delegate(mut self, delegate: Delegate) -> Self {
        self.delegates.push(delegate);
        self
    }
}

/// Builder helper for activated abilities
#[derive(Debug)]
pub struct ActivatedAbility {
    text: Vec<TextElement>,
    cost: Cost<AbilityId>,
    target_requirement: TargetRequirement<AbilityId>,
    delegates: Vec<Delegate>,
}

impl ActivatedAbility {
    pub fn new(text: Vec<TextElement>, cost: Cost<AbilityId>) -> Self {
        Self { text, cost, target_requirement: TargetRequirement::None, delegates: vec![] }
    }

    pub fn target_requirement(mut self, requirement: TargetRequirement<AbilityId>) -> Self {
        self.target_requirement = requirement;
        self
    }

    pub fn delegate(mut self, delegate: Delegate) -> Self {
        self.delegates.push(delegate);
        self
    }

    pub fn build(self) -> Ability {
        Ability {
            ability_type: AbilityType::Activated {
                cost: self.cost,
                target_requirement: self.target_requirement,
            },
            text: self.text,
            delegates: self.delegates,
        }
    }
}

/// The Possible resonances of weapons and minions. Minions can only be
/// damaged by weapons from the same resonance, or by Prismatic weapons.
#[derive(Debug, Default, Clone, Copy)]
pub struct Resonance {
    pub mortal: bool,
    pub infernal: bool,
    pub astral: bool,
    pub prismatic: bool,
}

impl Resonance {
    pub fn mortal() -> Self {
        Self { mortal: true, ..Self::default() }
    }

    pub fn infernal() -> Self {
        Self { infernal: true, ..Self::default() }
    }

    pub fn astral() -> Self {
        Self { astral: true, ..Self::default() }
    }

    pub fn prismatic() -> Self {
        Self { prismatic: true, ..Self::default() }
    }

    pub fn with_mortal(mut self, mortal: bool) -> Self {
        self.mortal = mortal;
        self
    }

    pub fn with_infernal(mut self, infernal: bool) -> Self {
        self.infernal = infernal;
        self
    }

    pub fn with_astral(mut self, astral: bool) -> Self {
        self.astral = astral;
        self
    }

    pub fn with_prismatic(mut self, prismatic: bool) -> Self {
        self.prismatic = prismatic;
        self
    }

    /// Counts how many of mortal, infernal, and astral resonances are present
    /// here
    pub fn basic_resonance_count(self) -> u32 {
        (if self.mortal { 1 } else { 0 })
            + (if self.infernal { 1 } else { 0 })
            + (if self.astral { 1 } else { 0 })
    }
}

/// Predicate which provides additional restrictions on whether a card can
/// be played. If not specified, normal rules apply.
pub type CanPlay = fn(&GameState, CardId) -> bool;

/// Individual card configuration; properties which are not universal for all
/// cards
#[derive(Debug, Default)]
pub struct CardConfig {
    pub stats: CardStats,
    pub resonance: Option<Resonance>,
    /// Targeting requirements for this card, e.g. to target a room.
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

    pub fn is_artifact(&self) -> bool {
        self.card_type == CardType::Artifact
    }
}
