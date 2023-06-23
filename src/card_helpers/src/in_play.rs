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

use game_data::delegates::{
    DealtDamage, Delegate, EventDelegate, MutationFn, QueryDelegate, RaidEvent, Scope,
    TransformationFn,
};
use game_data::game::GameState;
use game_data::primitives::{
    ActionCount, CardId, HasRoomId, RaidId, RoomIdCrypts, RoomIdMarker, RoomIdSanctum, RoomIdVault,
    TurnNumber,
};

/// A delegate which triggers at dawn if a card is face up in play
pub fn at_dawn(mutation: MutationFn<TurnNumber>) -> Delegate {
    Delegate::Dawn(EventDelegate { requirement: crate::face_up_in_play, mutation })
}

/// A delegate which triggers at dusk if a card is face up in play
pub fn at_dusk(mutation: MutationFn<TurnNumber>) -> Delegate {
    Delegate::Dusk(EventDelegate { requirement: crate::face_up_in_play, mutation })
}

/// A delegate which triggers if a card is face up in play when damage is dealt.
pub fn on_damage(mutation: MutationFn<DealtDamage>) -> Delegate {
    Delegate::DealtDamage(EventDelegate { requirement: crate::face_up_in_play, mutation })
}

/// A `RequirementFn` which matches for face up in play cards and events
/// targeting a specific room.
pub fn in_play_with_room<M: RoomIdMarker>(
    game: &GameState,
    scope: Scope,
    data: &impl HasRoomId,
) -> bool {
    crate::face_up_in_play(game, scope, &data) && data.room_id() == M::room_id()
}

/// Delegate which fires when the 'access' phase of a raid begins.
pub fn on_raid_access_start(mutation: MutationFn<RaidId>) -> Delegate {
    Delegate::RaidAccessStart(EventDelegate { requirement: crate::face_up_in_play, mutation })
}

/// A delegate which fires when a card is face up & in play when a raid on the
/// sanctum ends in success
pub fn after_sanctum_accessed(mutation: MutationFn<RaidEvent>) -> Delegate {
    Delegate::RaidSuccess(EventDelegate {
        requirement: in_play_with_room::<RoomIdSanctum>,
        mutation,
    })
}

/// A delegate which fires when a card is face up & in play when a raid on the
/// vault ends in success
pub fn after_vault_accessed(mutation: MutationFn<RaidEvent>) -> Delegate {
    Delegate::RaidSuccess(EventDelegate { requirement: in_play_with_room::<RoomIdVault>, mutation })
}

/// A delegate which fires when a card is face up & in play when a raid on the
/// crypts ends in success
pub fn after_crypts_accessed(mutation: MutationFn<RaidEvent>) -> Delegate {
    Delegate::RaidSuccess(EventDelegate {
        requirement: in_play_with_room::<RoomIdCrypts>,
        mutation,
    })
}

/// A delegate which fires when a card is face up & in play when a raid ends in
/// success
pub fn after_room_accessed(mutation: MutationFn<RaidEvent>) -> Delegate {
    Delegate::RaidSuccess(EventDelegate { requirement: crate::face_up_in_play, mutation })
}

/// A delegate which fires when a card is face up & in play when a raid on the
/// vault is selecting cards to access.
pub fn vault_access_selected(mutation: MutationFn<RaidEvent>) -> Delegate {
    Delegate::RaidAccessSelected(EventDelegate {
        requirement: in_play_with_room::<RoomIdVault>,
        mutation,
    })
}

/// A delegate which intercepts queries for the action costs of cards while its
/// parent is face up and in play.
pub fn on_query_action_cost(transformation: TransformationFn<CardId, ActionCount>) -> Delegate {
    Delegate::ActionCost(QueryDelegate { requirement: crate::face_up_in_play, transformation })
}
