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

use core_data::game_primitives::{AttackValue, CardId, HasAbilityId, HasCardId};
use game_data::delegate_data::{
    AbilityActivated, CanActivateAbility, CardPlayed, Delegate, DiscardedCard, EventDelegate,
    MutationFn, QueryDelegate, RaidEvent, Scope, TransformationFn, UsedWeapon,
};
use game_data::flag_data::Flag;
use game_data::game_state::GameState;
use game_data::prompt_data::{AbilityPromptSource, GamePrompt};

/// A RequirementFn which restricts delegates to only listen to events for their
/// own card.
pub fn card(_game: &GameState, scope: Scope, card_id: &impl HasCardId) -> bool {
    scope.card_id() == card_id.card_id()
}

/// A RequirementFn which restricts delegates to only listen to events for their
/// own ability.
pub fn ability(_game: &GameState, scope: Scope, ability_id: &impl HasAbilityId) -> bool {
    scope.ability_id() == ability_id.ability_id()
}

/// A delegate which triggers when this card is played
pub fn on_played(mutation: MutationFn<CardPlayed>) -> Delegate {
    Delegate::PlayCard(EventDelegate { requirement: card, mutation })
}

pub fn prompt(
    transformation: TransformationFn<AbilityPromptSource, Option<GamePrompt>>,
) -> Delegate {
    Delegate::ShowPrompt(QueryDelegate { requirement: ability, transformation })
}

/// A delegate which triggers when this card leaves play
pub fn on_leaves_play(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::LeaveArena(EventDelegate { requirement: card, mutation })
}

/// A [Delegate] which triggers when an ability is activated
pub fn on_activated(mutation: MutationFn<AbilityActivated>) -> Delegate {
    Delegate::ActivateAbility(EventDelegate { requirement: ability, mutation })
}

/// A [Delegate] which controls whether an ability can be activated.
pub fn can_activate(transformation: TransformationFn<CanActivateAbility, Flag>) -> Delegate {
    Delegate::CanActivateAbility(QueryDelegate { requirement: ability, transformation })
}

/// A minion combat delegate
pub fn combat(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::MinionCombatAbility(EventDelegate { requirement: card, mutation })
}

/// A delegate which triggers when this card is moved from a deck *or* hand to a
/// discard pile.
pub fn on_discarded(mutation: MutationFn<DiscardedCard>) -> Delegate {
    Delegate::DiscardCard(EventDelegate { requirement: card, mutation })
}

/// A delegate which triggers when this weapon is used
pub fn on_weapon_used(mutation: MutationFn<RaidEvent<UsedWeapon>>) -> Delegate {
    Delegate::UsedWeapon(EventDelegate {
        requirement: |_, s, used| s.card_id() == used.data.weapon_id,
        mutation,
    })
}

/// A delegate when a card is scored by the Covenant player
pub fn on_scored_by_covenant(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::CovenantScoreCard(EventDelegate { requirement: card, mutation })
}

/// A delegate when a card is scored by the Riftcaller player
pub fn on_scored_by_riftcaller(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::RiftcallerScoreCard(EventDelegate { requirement: card, mutation })
}

/// A delegate which prevents a card from being able to be played
pub fn can_play(transformation: TransformationFn<CardId, Flag>) -> Delegate {
    Delegate::CanPlayCard(QueryDelegate { requirement: card, transformation })
}

/// A delegate which prevents a card from being able to be evaded
pub fn can_evade(transformation: TransformationFn<CardId, Flag>) -> Delegate {
    Delegate::CanEvadeMinion(QueryDelegate { requirement: card, transformation })
}

/// A delegate which modifies this card's base attack value
pub fn base_attack(transformation: TransformationFn<CardId, AttackValue>) -> Delegate {
    Delegate::BaseAttack(QueryDelegate { requirement: card, transformation })
}
