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
    ActionCount, CardId, CurseCount, HasRoomId, ManaValue, RaidId, RoomIdCrypt, RoomIdMarker,
    RoomIdSanctum, RoomIdVault, Side, TurnNumber,
};
use game_data::card_definition::Resonance;
use game_data::delegate_data::{
    AccessEvent, CardPlayed, CardStatusMarker, DealtDamage, Delegate, EventDelegate, Flag,
    ManaLostToOpponentAbility, MutationFn, QueryDelegate, RaidEvent, Scope, ScoreCard,
    TransformationFn,
};
use game_data::game_state::GameState;
use game_data::raid_data::PopulateAccessPromptSource;

use crate::{delegates, requirements};

/// A delegate which triggers at dawn if a card is face up in play
pub fn at_dawn(mutation: MutationFn<TurnNumber>) -> Delegate {
    Delegate::Dawn(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which triggers at dusk if a card is face up in play
pub fn at_dusk(mutation: MutationFn<TurnNumber>) -> Delegate {
    Delegate::Dusk(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which triggers when any card is played if this card is face up
/// and in play
pub fn on_card_played(mutation: MutationFn<CardPlayed>) -> Delegate {
    Delegate::PlayCard(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which triggers if a card is face up in play before damage is
/// dealt.
pub fn on_will_deal_damage(mutation: MutationFn<DealtDamage>) -> Delegate {
    Delegate::WillDealDamage(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which triggers if a card is face up in play when damage is dealt.
pub fn on_damage(mutation: MutationFn<DealtDamage>) -> Delegate {
    Delegate::DealtDamage(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which triggers if a card is face up in play before a curse is
/// given
pub fn on_will_receive_curses(mutation: MutationFn<CurseCount>) -> Delegate {
    Delegate::WillReceiveCurses(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which triggers if a card is face up in play when one or more
/// curses are received.
pub fn on_curse(mutation: MutationFn<CurseCount>) -> Delegate {
    Delegate::CursesReceived(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A `RequirementFn` which matches for face up in play cards and events
/// targeting a specific room.
pub fn in_play_with_room<M: RoomIdMarker>(
    game: &GameState,
    scope: Scope,
    data: &impl HasRoomId,
) -> bool {
    requirements::face_up_in_play(game, scope, &data) && data.room_id() == M::room_id()
}

/// Delegate which fires when a raid starts
pub fn on_raid_started(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidStart(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which fires when a card is face up & in play when a minion is
/// summoned.
pub fn on_minion_summoned(mutation: MutationFn<CardId>) -> Delegate {
    delegates::on_minion_summoned(requirements::face_up_in_play, mutation)
}

/// A delegate which fires when a card is face up & in play when a minion is
/// approached.
pub fn on_minion_approached(mutation: MutationFn<RaidEvent<CardId>>) -> Delegate {
    delegates::on_minion_approached(requirements::face_up_in_play, mutation)
}

/// Delegate which fires when the 'access' phase of a raid begins.
pub fn on_raid_access_start(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidAccessStart(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid
/// accesses the sanctum
pub fn on_sanctum_access_start(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidAccessStart(EventDelegate {
        requirement: in_play_with_room::<RoomIdSanctum>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid
/// accesses the vault
pub fn on_vault_access_start(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidAccessStart(EventDelegate {
        requirement: in_play_with_room::<RoomIdVault>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid
/// accesses the crypt
pub fn on_crypt_access_start(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidAccessStart(EventDelegate {
        requirement: in_play_with_room::<RoomIdCrypt>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid on the
/// sanctum ends in success
pub fn after_sanctum_accessed(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidSuccess(EventDelegate {
        requirement: in_play_with_room::<RoomIdSanctum>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid on the
/// vault ends in success
pub fn after_vault_accessed(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidSuccess(EventDelegate { requirement: in_play_with_room::<RoomIdVault>, mutation })
}

/// A delegate which fires when its card is face up & in play when a raid on the
/// crypt ends in success
pub fn after_crypt_accessed(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidSuccess(EventDelegate { requirement: in_play_with_room::<RoomIdCrypt>, mutation })
}

/// A delegate which fires when its card is face up & in play when a raid's
/// access phase ends.
pub fn on_raid_access_end(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidAccessEnd(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which fires when its card is face up & in play when a raid ends
/// in success
pub fn on_raid_success(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidSuccess(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which fires when its card is face up & in play when a raid on the
/// vault is selecting cards to access.
pub fn vault_access_selected(mutation: MutationFn<RaidEvent<()>>) -> Delegate {
    Delegate::RaidAccessSelected(EventDelegate {
        requirement: in_play_with_room::<RoomIdVault>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a card is
/// scored
pub fn on_card_scored(mutation: MutationFn<ScoreCard>) -> Delegate {
    Delegate::ScoreCard(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which fires when its card is face up & in play when a card is
/// sacrificed
pub fn on_card_sacrificed(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::CardSacrificed(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which fires when its card is face up & in play when a card is
/// moved to a discard pile
pub fn on_card_moved_to_discard_pile(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::MoveToDiscardPile(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a card is
/// revealed by a card ability.
pub fn on_card_revealed(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::CardRevealed(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which fires when its card is face up & in play when one player
/// loses mana to an opponent ability.
pub fn on_mana_lost_to_opponent_ability(
    mutation: MutationFn<ManaLostToOpponentAbility>,
) -> Delegate {
    Delegate::ManaLostToOpponentAbility(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when one player
/// loses action points during a raid.
pub fn on_action_points_lost_during_raid(mutation: MutationFn<Side>) -> Delegate {
    Delegate::ActionPointsLostDuringRaid(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a player
/// is about to draw cards.
pub fn on_will_draw_cards(mutation: MutationFn<Side>) -> Delegate {
    delegates::on_will_draw_cards(requirements::face_up_in_play, mutation)
}

/// A delegate which fires when its card is face up & in play when a player
/// draws cards via an ability.
pub fn on_draw_cards_via_ability(mutation: MutationFn<Side>) -> Delegate {
    Delegate::DrawCardsViaAbility(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when the raid
/// access prompt is being populated.
pub fn on_will_populate_access_prompt(
    mutation: MutationFn<AccessEvent<PopulateAccessPromptSource>>,
) -> Delegate {
    Delegate::WillPopulateAccessPrompt(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which transforms the sanctum access count when a card is face up
/// & in play
pub fn on_query_sanctum_access_count(transformation: TransformationFn<RaidId, u32>) -> Delegate {
    delegates::sanctum_access_count(requirements::face_up_in_play, transformation)
}

/// A delegate which transforms the vault access count when a card is face up
/// & in play
pub fn on_query_vault_access_count(transformation: TransformationFn<RaidId, u32>) -> Delegate {
    delegates::vault_access_count(requirements::face_up_in_play, transformation)
}

/// A delegate which intercepts queries for the action costs of cards while its
/// parent is face up and in play.
pub fn on_query_action_cost(transformation: TransformationFn<CardId, ActionCount>) -> Delegate {
    Delegate::ActionCost(QueryDelegate {
        requirement: requirements::face_up_in_play,
        transformation,
    })
}

/// A delegate which intercepts queries for the mana costs of cards while its
/// parent is face up and in play.
pub fn on_query_mana_cost(transformation: TransformationFn<CardId, Option<ManaValue>>) -> Delegate {
    Delegate::ManaCost(QueryDelegate { requirement: requirements::face_up_in_play, transformation })
}

/// A delegate which intercepts queries for a card's [Resonance] when a card is
/// face up & in play.
pub fn on_query_resonance(transformation: TransformationFn<CardId, Resonance>) -> Delegate {
    Delegate::Resonance(QueryDelegate {
        requirement: requirements::face_up_in_play,
        transformation,
    })
}

/// A delegate which intercepts queries for a card's [CardStatusMarker]s when
/// its card is face up & in play.
pub fn on_query_card_status_markers(
    transformation: TransformationFn<CardId, Vec<CardStatusMarker>>,
) -> Delegate {
    delegates::on_query_card_status_markers(requirements::face_up_in_play, transformation)
}

/// A delegate which intercepts queries for whether an accessed card can be
/// scored while its card is face up & in play.
pub fn can_score_accessed_card(
    transformation: TransformationFn<AccessEvent<CardId>, Flag>,
) -> Delegate {
    delegates::can_score_accessed_card(requirements::face_up_in_play, transformation)
}
