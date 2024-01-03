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

use core_data::game_primitives::{
    AbilityId, AttackValue, CardId, HasAbilityId, HasCardId, TurnNumber,
};
use game_data::card_definition::Cost;
use game_data::delegate_data::{
    AbilityActivated, CanActivateAbility, CardEncounter, CardPlayed, DiscardedCard, EventDelegate,
    GameDelegate, MutationFn, QueryDelegate, RaidEvent, Scope, TransformationFn, UsedWeapon,
};
use game_data::flag_data::Flag;
use game_data::game_state::GameState;
use game_data::prompt_data::{AbilityPromptSource, GamePrompt};

use crate::{delegates, history, requirements};

/// A RequirementFn which restricts delegates to only listen to events for their
/// own card.
pub fn card(_: &GameState, scope: Scope, card_id: &impl HasCardId) -> bool {
    scope.card_id() == card_id.card_id()
}

/// A RequirementFn which restricts delegates to only listen to events for their
/// own ability.
pub fn ability(_: &GameState, scope: Scope, ability_id: &impl HasAbilityId) -> bool {
    scope.ability_id() == ability_id.ability_id()
}

/// A delegate which triggers when this card is played
pub fn on_played(mutation: MutationFn<CardPlayed>) -> GameDelegate {
    GameDelegate::PlayCard(EventDelegate { requirement: card, mutation })
}

/// Implements a callback to build a [GamePrompt] for this card when one is
/// requested via `prompts::push();`
pub fn prompt(
    transformation: TransformationFn<AbilityPromptSource, Option<GamePrompt>>,
) -> GameDelegate {
    GameDelegate::ShowPrompt(QueryDelegate { requirement: ability, transformation })
}

/// A delegate which triggers when this card leaves play
pub fn on_leaves_play(mutation: MutationFn<CardId>) -> GameDelegate {
    GameDelegate::LeaveArena(EventDelegate { requirement: card, mutation })
}

/// A [GameDelegate] which triggers when an ability is activated
pub fn on_activated(mutation: MutationFn<AbilityActivated>) -> GameDelegate {
    GameDelegate::ActivateAbility(EventDelegate { requirement: ability, mutation })
}

/// A [GameDelegate] which controls whether an ability can be activated.
pub fn can_activate(transformation: TransformationFn<CanActivateAbility, Flag>) -> GameDelegate {
    GameDelegate::CanActivateAbility(QueryDelegate { requirement: ability, transformation })
}

/// A [GameDelegate] which allows an ability to be activated once per turn.
pub fn once_per_turn() -> GameDelegate {
    GameDelegate::CanActivateAbility(QueryDelegate {
        requirement: ability,
        transformation: |g, s, _, flag| {
            flag.add_constraint(
                history::ability_activations_this_turn(g, s.ability_id()).next().is_none(),
            )
        },
    })
}

/// A minion combat delegate
pub fn combat(mutation: MutationFn<CardId>) -> GameDelegate {
    GameDelegate::MinionCombatAbility(EventDelegate { requirement: card, mutation })
}

/// A delegate which triggers at dawn
pub fn at_dawn(mutation: MutationFn<TurnNumber>) -> GameDelegate {
    GameDelegate::Dawn(EventDelegate { requirement: requirements::always, mutation })
}

/// A delegate which triggers at dusk
pub fn at_dusk(mutation: MutationFn<TurnNumber>) -> GameDelegate {
    GameDelegate::Dusk(EventDelegate { requirement: requirements::always, mutation })
}

/// A delegate which triggers when this card is moved from a deck *or* hand to a
/// discard pile.
pub fn on_discarded(mutation: MutationFn<DiscardedCard>) -> GameDelegate {
    GameDelegate::DiscardCard(EventDelegate { requirement: card, mutation })
}

/// A delegate which triggers when this weapon is used
pub fn on_weapon_used(mutation: MutationFn<RaidEvent<UsedWeapon>>) -> GameDelegate {
    GameDelegate::UsedWeapon(EventDelegate {
        requirement: |_, s, used| s.card_id() == used.data.weapon_id,
        mutation,
    })
}

/// A delegate which triggers when a card is scored by the Covenant player
pub fn on_scored_by_covenant(mutation: MutationFn<CardId>) -> GameDelegate {
    GameDelegate::CovenantScoreCard(EventDelegate { requirement: card, mutation })
}

/// A delegate which triggers when a card is scored by the Riftcaller player
pub fn on_scored_by_riftcaller(mutation: MutationFn<CardId>) -> GameDelegate {
    GameDelegate::RiftcallerScoreCard(EventDelegate { requirement: card, mutation })
}

/// A delegate which triggers when a card's CardSelector prompt is submitted.
pub fn on_card_selector_submitted(mutation: MutationFn<AbilityId>) -> GameDelegate {
    GameDelegate::CardSelectorSubmitted(EventDelegate { requirement: card, mutation })
}

/// A delegate which computes a custom cost to score a scheme card the
/// Riftcaller is accessing.
pub fn score_accessed_card_cost(
    transformation: TransformationFn<CardId, Cost<CardId>>,
) -> GameDelegate {
    GameDelegate::ScoreAccessedCardCost(QueryDelegate { requirement: card, transformation })
}

/// A delegate which prevents a card from being able to be played
pub fn can_play(transformation: TransformationFn<CardId, Flag>) -> GameDelegate {
    delegates::can_play_card(card, transformation)
}

/// A delegate which prevents a weapon from being able to be used
pub fn can_use_weapon(transformation: TransformationFn<CardEncounter, Flag>) -> GameDelegate {
    GameDelegate::CanUseWeapon(QueryDelegate {
        requirement: |_, s, encounter| encounter.weapon_id == s.card_id(),
        transformation,
    })
}

/// A delegate which prevents a card from being able to be evaded
pub fn can_evade(transformation: TransformationFn<CardId, Flag>) -> GameDelegate {
    GameDelegate::CanEvadeMinion(QueryDelegate { requirement: card, transformation })
}

/// A delegate which modifies this card's base attack value
pub fn base_attack(transformation: TransformationFn<CardId, AttackValue>) -> GameDelegate {
    GameDelegate::BaseAttack(QueryDelegate { requirement: card, transformation })
}

/// A delegate which modifies whether this card is a 'slow' weapon (paying
/// double for shield costs)
pub fn is_slow_weapon(transformation: TransformationFn<CardId, bool>) -> GameDelegate {
    GameDelegate::IsSlowWeapon(QueryDelegate { requirement: card, transformation })
}
