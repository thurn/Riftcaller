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
pub mod delegates;
pub mod effects;
pub mod history;
pub mod in_play;
pub mod projects;
pub mod raids;
pub mod requirements;
pub mod show_prompt;
pub mod text_macro;
pub mod this;

use game_data::card_definition::{AbilityType, Cost, TargetRequirement};
use game_data::card_state::CardPosition;
use game_data::delegate_data::{
    Delegate, EventDelegate, MutationFn, QueryDelegate, RaidEvent, RaidOutcome, RequirementFn,
    Scope, TransformationFn, UsedWeapon,
};
use game_data::game_actions::PromptChoice;
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::primitives::{
    ActionCount, CardId, HasAbilityId, HasCardId, HealthValue, ManaValue, RoomId, Side,
};
pub use game_data::text::TextToken::*;
use game_data::text::{TextElement, TextToken};
use rules::mana;
use rules::mana::ManaPurpose;

pub fn trigger_text(name: TextToken, effect: Vec<TextElement>) -> Vec<TextElement> {
    vec![TextElement::NamedTrigger(name, effect)]
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

/// An [AbilityType] for an ability which costs 1 action and has no target.
pub fn activate_for_action() -> AbilityType {
    AbilityType::Activated { cost: costs::actions(1), target_requirement: TargetRequirement::None }
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

pub fn when_unveiled(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::UnveilCard(EventDelegate { requirement: this_card, mutation })
}

/// A minion delegate which triggers when it is encountered
pub fn on_encountered(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::EncounterMinion(EventDelegate { requirement: this_card, mutation })
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
    requirement: RequirementFn<RaidEvent<UsedWeapon>>,
    mutation: MutationFn<RaidEvent<UsedWeapon>>,
) -> Delegate {
    Delegate::UsedWeapon(EventDelegate { requirement, mutation })
}

/// Delegate which fires when its card is accessed
pub fn on_accessed(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::CardAccess(EventDelegate { requirement: this_card, mutation })
}

/// A delegate which fires when a raid ends in any way (except the game ending).
pub fn on_raid_ended(
    requirement: RequirementFn<RaidEvent<RaidOutcome>>,
    mutation: MutationFn<RaidEvent<RaidOutcome>>,
) -> Delegate {
    Delegate::RaidEnd(EventDelegate { requirement, mutation })
}

/// A delegate which fires when a raid ends in success
pub fn on_raid_success(
    requirement: RequirementFn<RaidEvent<()>>,
    mutation: MutationFn<RaidEvent<()>>,
) -> Delegate {
    Delegate::RaidSuccess(EventDelegate { requirement, mutation })
}

/// Delegate which transforms how a minion's health is calculated
pub fn on_calculate_health(transformation: TransformationFn<CardId, HealthValue>) -> Delegate {
    Delegate::HealthValue(QueryDelegate { requirement: this_card, transformation })
}

/// A [PromptChoice] to end the current raid.
pub fn end_raid_prompt() -> Option<PromptChoice> {
    Some(PromptChoice::new().effect(GameEffect::EndRaid))
}

/// A [PromptChoice] for the `side` player to lose mana
pub fn lose_mana_prompt(game: &GameState, side: Side, amount: ActionCount) -> Option<PromptChoice> {
    if mana::get(game, side, ManaPurpose::PayForTriggeredAbility) >= amount {
        Some(PromptChoice::new().effect(GameEffect::LoseMana(side, amount)))
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
        Some(PromptChoice::new().effect(GameEffect::LoseActions(side, amount)))
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
        Some(PromptChoice::new().effect(GameEffect::TakeDamage(ability_id.ability_id(), amount)))
    } else {
        None
    }
}
