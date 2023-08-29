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

//! Helpers for defining card behaviors. This file is intended be be used via
//! wildcard import in card definition files.

pub mod abilities;
pub mod costs;
pub mod history;
pub mod in_play;
pub mod projects;
pub mod text_macro;
pub mod this;

use anyhow::Result;
use game_data::card_definition::{
    Ability, AbilityType, AttackBoost, CardStats, Cost, CustomCost, SchemePoints, SpecialEffects,
    TargetRequirement,
};
use game_data::card_state::CardPosition;
use game_data::delegates::{
    AbilityActivated, CardPlayed, Delegate, EventDelegate, MutationFn, QueryDelegate, RaidEnded,
    RaidEvent, RequirementFn, Scope, TransformationFn, UsedWeapon,
};
use game_data::game::GameState;
use game_data::game_actions::{CardTarget, PromptChoice};
use game_data::game_effect::GameEffect;
use game_data::primitives::{
    AbilityId, ActionCount, AttackValue, CardId, HasAbilityId, HasCardId, HealthValue, ManaValue,
    RaidId, RoomId, Side,
};
use game_data::special_effects::Projectile;
pub use game_data::text::TextToken::*;
use game_data::text::{TextElement, TextToken};
use game_data::updates::{GameUpdate, InitiatedBy};
use game_data::utils;
use rules::mana;
use rules::mana::ManaPurpose;

pub fn trigger_text(name: TextToken, effect: Vec<TextElement>) -> Vec<TextElement> {
    vec![TextElement::NamedTrigger(name, effect)]
}

pub fn encounter_ability_text(
    cost: Vec<TextElement>,
    effect: Vec<TextElement>,
) -> Vec<TextElement> {
    vec![TextElement::EncounterAbility { cost, effect }]
}

pub fn reminder(text: &'static str) -> TextElement {
    TextElement::Reminder(text.to_string())
}

/// An ability which only exists to add text to a card.
pub fn text_only_ability(text: Vec<TextElement>) -> Ability {
    Ability { text, ability_type: AbilityType::TextOnly, delegates: vec![] }
}

/// A [Cost] which requires no mana and `actions` action points.
pub fn actions(actions: ActionCount) -> Cost<AbilityId> {
    Cost { mana: None, actions, custom_cost: None }
}

/// Provides the cost for a card, with 1 action point required and `mana` mana
/// points
pub fn cost(mana: ManaValue) -> Cost<CardId> {
    Cost { mana: Some(mana), actions: 1, custom_cost: None }
}

/// [Cost] for a scheme card
pub fn scheme_cost() -> Cost<CardId> {
    Cost { mana: None, actions: 1, custom_cost: None }
}

/// [Cost] for a riftcaller card
pub fn riftcaller_cost() -> Cost<CardId> {
    Cost { mana: None, actions: 0, custom_cost: None }
}

/// A [CustomCost] which allows an ability to be activated once per turn.
///
/// Stores turn data in ability state. Never returns `None`.
pub fn once_per_turn_cost() -> Option<CustomCost<AbilityId>> {
    Some(CustomCost {
        can_pay: |game, ability_id| {
            utils::is_false(|| Some(game.ability_state(ability_id)?.turn? == game.info.turn))
        },
        pay: |game, ability_id| {
            game.ability_state_mut(ability_id).turn = Some(game.info.turn);
            Ok(())
        },
        description: None,
    })
}

/// Creates a standard [Ability] with a single [Delegate].
pub fn simple_ability(text: Vec<TextElement>, delegate: Delegate) -> Ability {
    Ability { text, ability_type: AbilityType::Standard, delegates: vec![delegate] }
}

/// An [AbilityType] for an ability which costs 1 action and has no target.
pub fn activate_for_action() -> AbilityType {
    AbilityType::Activated(actions(1), TargetRequirement::None)
}

/// RequirementFn which always returns true
pub fn always<T>(_: &GameState, _: Scope, _: &T) -> bool {
    true
}

/// RequirementFn which checks if the [Side] parameter is [Side::Champion]
pub fn side_is_champion(_: &GameState, _: Scope, side: &Side) -> bool {
    *side == Side::Champion
}

/// RequirementFn which checks if the [RoomId] parameter is [RoomId::Sanctum]
pub fn room_is_sanctum(_: &GameState, _: Scope, room_id: &RoomId) -> bool {
    *room_id == RoomId::Sanctum
}

/// RequirementFn which checks if the [RoomId] parameter is [RoomId::Vault]
pub fn room_is_vault(_: &GameState, _: Scope, room_id: &RoomId) -> bool {
    *room_id == RoomId::Vault
}

/// RequirementFn which checks if the [RoomId] parameter is [RoomId::Crypts]
pub fn room_is_crypts(_: &GameState, _: Scope, room_id: &RoomId) -> bool {
    *room_id == RoomId::Crypts
}

/// RequirementFn that this delegate's card is currently face up & in play
pub fn face_up_in_play<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    let card = game.card(scope.card_id());
    card.is_face_up() && card.position().in_play()
}

/// RequirementFn that this delegate's card is currently face down & in play
pub fn face_down_in_play<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    let card = game.card(scope.card_id());
    card.is_face_down() && card.position().in_play()
}

/// RequirementFn that this delegate's card is currently in its owner's score
/// pile
pub fn scored_by_owner<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    game.card(scope.card_id()).position() == CardPosition::Scored(scope.side())
}

/// A RequirementFn which restricts delegates to only listen to events for their
/// own card.
pub fn this_card(_game: &GameState, scope: Scope, card_id: &impl HasCardId) -> bool {
    scope.card_id() == card_id.card_id()
}

/// A RequirementFn which restricts delegates to only listen to events for their
/// own ability.
pub fn this_ability(_game: &GameState, scope: Scope, ability_id: &impl HasAbilityId) -> bool {
    scope.ability_id() == ability_id.ability_id()
}

/// A RequirementFn which checks if the current `raid_id` matches the stored
/// [RaidId] for this `scope`.
pub fn matching_raid<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    utils::is_true(|| {
        Some(game.ability_state(scope.ability_id())?.raid_id? == game.info.raid.as_ref()?.raid_id)
    })
}

/// Predicate checking if a room is an inner room
pub fn is_inner_room(room_id: RoomId) -> bool {
    room_id == RoomId::Vault || room_id == RoomId::Sanctum || room_id == RoomId::Crypts
}

/// Pushes a [GameUpdate] indicating the ability represented by [Scope] should
/// have a trigger animation shown in the UI.
pub fn alert(game: &mut GameState, scope: Scope) {
    game.record_update(|| GameUpdate::AbilityTriggered(scope.ability_id()));
}

/// Invokes [alert] if the provided `number` is not zero.
pub fn alert_if_nonzero(game: &mut GameState, scope: Scope, number: u32) {
    if number > 0 {
        alert(game, scope);
    }
}

/// A delegate which triggers when a card is cast
pub fn on_cast(mutation: MutationFn<CardPlayed>) -> Delegate {
    Delegate::CastCard(EventDelegate { requirement: this_card, mutation })
}

/// A [Delegate] which triggers when an ability is activated
pub fn on_activated(mutation: MutationFn<AbilityActivated>) -> Delegate {
    Delegate::ActivateAbility(EventDelegate { requirement: this_ability, mutation })
}

pub fn when_unveiled(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::UnveilCard(EventDelegate { requirement: this_card, mutation })
}

/// A minion delegate which triggers when it is encountered
pub fn on_encountered(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::EncounterMinion(EventDelegate { requirement: this_card, mutation })
}

/// Delegate to supply supplemental minion actions when encountered.
pub fn minion_combat_actions(
    transformation: TransformationFn<CardId, Vec<Option<PromptChoice>>>,
) -> Delegate {
    Delegate::MinionCombatActions(QueryDelegate { requirement: this_card, transformation })
}

/// A minion combat delegate
pub fn combat(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::MinionCombatAbility(EventDelegate { requirement: this_card, mutation })
}

/// A delegate when a card is scored
pub fn on_overlord_score(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::OverlordScoreCard(EventDelegate { requirement: this_card, mutation })
}

/// Delegate which fires when a weapon is used
pub fn on_weapon_used(
    requirement: RequirementFn<UsedWeapon>,
    mutation: MutationFn<UsedWeapon>,
) -> Delegate {
    Delegate::UsedWeapon(EventDelegate { requirement, mutation })
}

/// Delegate which fires when its card is accessed
pub fn on_accessed(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::CardAccess(EventDelegate { requirement: this_card, mutation })
}

/// A delegate which fires when a raid ends in any way (except the game ending).
pub fn on_raid_ended(
    requirement: RequirementFn<RaidEnded>,
    mutation: MutationFn<RaidEnded>,
) -> Delegate {
    Delegate::RaidEnd(EventDelegate { requirement, mutation })
}

/// A delegate which fires when a raid ends in success
pub fn on_raid_success(
    requirement: RequirementFn<RaidEvent>,
    mutation: MutationFn<RaidEvent>,
) -> Delegate {
    Delegate::RaidSuccess(EventDelegate { requirement, mutation })
}

/// Delegate which transforms how a minion's health is calculated
pub fn on_calculate_health(transformation: TransformationFn<CardId, HealthValue>) -> Delegate {
    Delegate::HealthValue(QueryDelegate { requirement: this_card, transformation })
}

pub fn add_vault_access<const N: u32>(requirement: RequirementFn<RaidId>) -> Delegate {
    Delegate::VaultAccessCount(QueryDelegate {
        requirement,
        transformation: |_, _, _, current| current + N,
    })
}

pub fn add_sanctum_access<const N: u32>(requirement: RequirementFn<RaidId>) -> Delegate {
    Delegate::SanctumAccessCount(QueryDelegate {
        requirement,
        transformation: |_, _, _, current| current + N,
    })
}

/// Helper to create a [CardStats] with the given base [AttackValue]
pub fn base_attack(base_attack: AttackValue) -> CardStats {
    CardStats { base_attack: Some(base_attack), ..CardStats::default() }
}

/// Helper to create a [CardStats] with the given base [AttackValue] and
/// [AttackBoost]
pub fn attack(base_attack: AttackValue, boost: AttackBoost) -> CardStats {
    CardStats { base_attack: Some(base_attack), attack_boost: Some(boost), ..CardStats::default() }
}

/// Helper to create a [CardStats] with the given [HealthValue]
pub fn health(health: HealthValue) -> CardStats {
    CardStats { health: Some(health), ..CardStats::default() }
}

/// Helper to create a [CardStats] with the given [SchemePoints].
pub fn scheme_points(points: SchemePoints) -> CardStats {
    CardStats { scheme_points: Some(points), ..CardStats::default() }
}

/// Initiates a raid on the `target` room and stores the raid ID as ability
/// state.
pub fn initiate_raid(game: &mut GameState, scope: Scope, target: CardTarget) -> Result<()> {
    initiate_raid_with_callback(game, scope, target, |_, _| {})
}

/// Initiates a raid on the `target` room and stores the raid ID as ability
/// state.
///
/// Invokes `on_begin` as soon as a [RaidId] is available.
pub fn initiate_raid_with_callback(
    game: &mut GameState,
    scope: Scope,
    target: CardTarget,
    on_begin: impl Fn(&mut GameState, RaidId),
) -> Result<()> {
    raids::initiate(game, target.room_id()?, InitiatedBy::Card, |game, raid_id| {
        game.ability_state_mut(scope.ability_id()).raid_id = Some(raid_id);
        on_begin(game, raid_id);
    })
}

/// Invokes `function` at most once per turn.
///
/// Stores ability state to track the last-invoked turn number
pub fn once_per_turn<T>(
    game: &mut GameState,
    scope: Scope,
    data: &T,
    function: MutationFn<T>,
) -> Result<()> {
    if utils::is_false(|| Some(game.ability_state(scope.ability_id())?.turn? == game.info.turn)) {
        save_turn(game, scope);
        function(game, scope, data)
    } else {
        Ok(())
    }
}

/// Stores the current turn as ability state for the provided `ability_id`.
pub fn save_turn(game: &mut GameState, ability_id: impl HasAbilityId) {
    game.ability_state_mut(ability_id.ability_id()).turn = Some(game.info.turn);
}

/// Helper to store the provided [RaidId] as ability state for this [Scope].
pub fn save_raid_id(
    game: &mut GameState,
    ability_id: impl HasAbilityId,
    raid_id: &RaidId,
) -> Result<()> {
    game.ability_state_mut(ability_id.ability_id()).raid_id = Some(*raid_id);
    Ok(())
}

/// Add `amount` to the stored mana in a card. Returns the new stored amount.
pub fn add_stored_mana(game: &mut GameState, card_id: CardId, amount: ManaValue) -> ManaValue {
    game.card_mut(card_id).data.stored_mana += amount;
    game.card(card_id).data.stored_mana
}

/// Creates a [SpecialEffects] to fire a given [Projectile].
pub fn projectile(projectile: Projectile) -> SpecialEffects {
    SpecialEffects { projectile: Some(projectile), additional_hit: None }
}

/// A [PromptChoice] to end the current raid.
pub fn end_raid_prompt(_: &GameState) -> Option<PromptChoice> {
    Some(PromptChoice::from_effect(GameEffect::EndRaid))
}

/// A [PromptChoice] for the `side` player to lose mana
pub fn lose_mana_prompt(game: &GameState, side: Side, amount: ActionCount) -> Option<PromptChoice> {
    if mana::get(game, side, ManaPurpose::PayForTriggeredAbility) >= amount {
        Some(PromptChoice::from_effect(GameEffect::LoseMana(side, amount)))
    } else {
        None
    }
}

/// A [PromptChoice] for the `side` player to lose action points.
pub fn lose_actions_prompt(
    game: &GameState,
    side: Side,
    amount: ActionCount,
) -> Option<PromptChoice> {
    if game.player(side).actions >= amount {
        Some(PromptChoice::from_effect(GameEffect::LoseActions(side, amount)))
    } else {
        None
    }
}

/// A [PromptChoice] for the Champion player to take damage if they are able
/// to without losing the game
pub fn take_damage_prompt(
    game: &GameState,
    ability_id: impl HasAbilityId,
    amount: u32,
) -> Option<PromptChoice> {
    if game.hand(Side::Champion).count() >= amount as usize {
        Some(PromptChoice::from_effect(GameEffect::TakeDamage(ability_id.ability_id(), amount)))
    } else {
        None
    }
}
