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
    AbilityId, ActionCount, CardId, CurseCount, HasRoomId, ManaValue, RaidId, Resonance, RoomId,
    RoomIdCrypt, RoomIdMarker, RoomIdSanctum, RoomIdVault, Side, TurnNumber,
};
use enumset::EnumSet;
use game_data::delegate_data::{
    AccessEvent, CardPlayed, CardStatusMarker, DealtDamage, EventDelegate, GameDelegate,
    ManaLostToOpponentAbility, MutationFn, QueryDelegate, RaidEvent, Scope, ScoreCard,
    TransformationFn,
};
use game_data::flag_data::{AbilityFlag, Flag};
use game_data::game_state::GameState;
use game_data::raid_data::{MinionDefeated, PopulateAccessPromptSource};

use crate::{delegates, requirements};

/// A delegate which triggers at dawn if a card is face up in play
pub fn at_dawn(mutation: MutationFn<TurnNumber>) -> GameDelegate {
    GameDelegate::Dawn(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which triggers at dusk if a card is face up in play
pub fn at_dusk(mutation: MutationFn<TurnNumber>) -> GameDelegate {
    GameDelegate::Dusk(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which triggers when a card enters a player's hand if this card is
/// face up in play
pub fn on_enter_hand(mutation: MutationFn<CardId>) -> GameDelegate {
    GameDelegate::EnterHand(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which triggers when any card is played if this card is face up
/// and in play
pub fn on_card_played(mutation: MutationFn<CardPlayed>) -> GameDelegate {
    GameDelegate::PlayCard(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which triggers when a player takes the basic action to draw a
/// card if this card is face up and in play
pub fn on_draw_card_action(mutation: MutationFn<Side>) -> GameDelegate {
    GameDelegate::DrawCardAction(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which triggers when a player takes the basic action to gain mana
/// if this card is face up and in play
pub fn on_gain_mana_action(mutation: MutationFn<Side>) -> GameDelegate {
    GameDelegate::GainManaAction(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which triggers when a player takes the basic action to progress a
/// room if this card is face up and in play
pub fn on_progress_card_action(mutation: MutationFn<RoomId>) -> GameDelegate {
    GameDelegate::ProgressCardAction(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which triggers if a card is face up in play before damage is
/// dealt.
pub fn on_will_deal_damage(mutation: MutationFn<AbilityId>) -> GameDelegate {
    GameDelegate::WillDealDamage(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which triggers if a card is face up in play when damage is dealt.
pub fn on_damage(mutation: MutationFn<DealtDamage>) -> GameDelegate {
    GameDelegate::DealtDamage(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which triggers if a card is face up in play before a curse is
/// given
pub fn on_will_receive_curses(mutation: MutationFn<CurseCount>) -> GameDelegate {
    GameDelegate::WillReceiveCurses(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which triggers if a card is face up in play when one or more
/// curses are received.
pub fn on_curse(mutation: MutationFn<CurseCount>) -> GameDelegate {
    GameDelegate::CursesReceived(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which triggers if a card is face up in play before one or more
/// cards are destroyed
pub fn on_will_destroy_cards(mutation: MutationFn<Vec<CardId>>) -> GameDelegate {
    GameDelegate::WillDestroyCards(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
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
pub fn on_raid_started(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidStart(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which fires when a card is face up & in play when a minion is
/// summoned.
pub fn on_minion_summoned(mutation: MutationFn<CardId>) -> GameDelegate {
    delegates::on_minion_summoned(requirements::face_up_in_play, mutation)
}

/// A delegate which fires when a card is face up & in play when a minion is
/// approached.
pub fn on_minion_approached(mutation: MutationFn<RaidEvent<CardId>>) -> GameDelegate {
    delegates::on_minion_approached(requirements::face_up_in_play, mutation)
}

/// A delegate which fires when a card is face up & in play when a minion is
/// defeated.
pub fn on_minion_defeated(mutation: MutationFn<MinionDefeated>) -> GameDelegate {
    delegates::on_minion_defeated(requirements::face_up_in_play, mutation)
}

/// A delegate which fires when a card is face up & in play when an ability is
/// about to end the current raid.
pub fn on_ability_will_end_raid(mutation: MutationFn<RaidEvent<AbilityId>>) -> GameDelegate {
    delegates::on_ability_will_end_raid(requirements::face_up_in_play, mutation)
}

/// Delegate which fires when the 'access' phase of a raid begins.
pub fn on_raid_access_start(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidAccessStart(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid
/// accesses the sanctum
pub fn on_sanctum_access_start(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidAccessStart(EventDelegate {
        requirement: in_play_with_room::<RoomIdSanctum>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid
/// accesses the vault
pub fn on_vault_access_start(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidAccessStart(EventDelegate {
        requirement: in_play_with_room::<RoomIdVault>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid
/// accesses the crypt
pub fn on_crypt_access_start(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidAccessStart(EventDelegate {
        requirement: in_play_with_room::<RoomIdCrypt>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid on the
/// sanctum ends in success
pub fn after_sanctum_accessed(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidSuccess(EventDelegate {
        requirement: in_play_with_room::<RoomIdSanctum>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid on the
/// vault ends in success
pub fn after_vault_accessed(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidSuccess(EventDelegate {
        requirement: in_play_with_room::<RoomIdVault>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid on the
/// crypt ends in success
pub fn after_crypt_accessed(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidSuccess(EventDelegate {
        requirement: in_play_with_room::<RoomIdCrypt>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid's
/// access phase ends.
pub fn on_raid_access_end(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidAccessEnd(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid ends
/// in success
pub fn on_raid_success(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidSuccess(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a raid on the
/// vault is selecting cards to access.
pub fn vault_access_selected(mutation: MutationFn<RaidEvent<()>>) -> GameDelegate {
    GameDelegate::RaidAccessSelected(EventDelegate {
        requirement: in_play_with_room::<RoomIdVault>,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when the Covenant
/// scores a card.
pub fn on_covenant_scored_card(mutation: MutationFn<CardId>) -> GameDelegate {
    GameDelegate::CovenantScoreCard(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when the
/// Riftcaller scores a card.
pub fn on_riftcaller_scored_card(mutation: MutationFn<CardId>) -> GameDelegate {
    GameDelegate::RiftcallerScoreCard(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a card is
/// scored
pub fn on_card_scored(mutation: MutationFn<ScoreCard>) -> GameDelegate {
    GameDelegate::ScoreCard(EventDelegate { requirement: requirements::face_up_in_play, mutation })
}

/// A delegate which fires when its card is face up & in play when a card is
/// razed
pub fn on_card_razed(mutation: MutationFn<AccessEvent<CardId>>) -> GameDelegate {
    delegates::on_card_razed(requirements::face_up_in_play, mutation)
}

/// A delegate which fires when its card is face up & in play when a card is
/// sacrificed
pub fn on_card_sacrificed(mutation: MutationFn<CardId>) -> GameDelegate {
    GameDelegate::CardSacrificed(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a card is
/// moved to a discard pile
pub fn on_card_moved_to_discard_pile(mutation: MutationFn<CardId>) -> GameDelegate {
    GameDelegate::MoveToDiscardPile(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a card is
/// revealed by a card ability.
pub fn on_card_revealed(mutation: MutationFn<CardId>) -> GameDelegate {
    GameDelegate::CardRevealed(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when one player
/// loses mana to an opponent ability.
pub fn on_mana_lost_to_opponent_ability(
    mutation: MutationFn<ManaLostToOpponentAbility>,
) -> GameDelegate {
    GameDelegate::ManaLostToOpponentAbility(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when one player
/// loses action points during a raid.
pub fn on_action_points_lost_during_raid(mutation: MutationFn<Side>) -> GameDelegate {
    GameDelegate::ActionPointsLostDuringRaid(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when a player
/// is about to draw cards.
pub fn on_will_draw_cards(mutation: MutationFn<Side>) -> GameDelegate {
    delegates::on_will_draw_cards(requirements::face_up_in_play, mutation)
}

/// A delegate which fires when its card is face up & in play when a player
/// draws cards via an ability.
pub fn on_draw_cards_via_ability(mutation: MutationFn<Side>) -> GameDelegate {
    GameDelegate::DrawCardsViaAbility(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which fires when its card is face up & in play when the raid
/// access prompt is being populated.
pub fn on_will_populate_access_prompt(
    mutation: MutationFn<AccessEvent<PopulateAccessPromptSource>>,
) -> GameDelegate {
    GameDelegate::WillPopulateAccessPrompt(EventDelegate {
        requirement: requirements::face_up_in_play,
        mutation,
    })
}

/// A delegate which transforms the sanctum access count when a card is face up
/// & in play
pub fn on_query_sanctum_access_count(
    transformation: TransformationFn<RaidId, u32>,
) -> GameDelegate {
    delegates::sanctum_access_count(requirements::face_up_in_play, transformation)
}

/// A delegate which transforms the vault access count when a card is face up
/// & in play
pub fn on_query_vault_access_count(transformation: TransformationFn<RaidId, u32>) -> GameDelegate {
    delegates::vault_access_count(requirements::face_up_in_play, transformation)
}

/// A delegate which intercepts queries for the action costs of cards while its
/// parent is face up and in play.
pub fn on_query_action_cost(transformation: TransformationFn<CardId, ActionCount>) -> GameDelegate {
    GameDelegate::ActionCost(QueryDelegate {
        requirement: requirements::face_up_in_play,
        transformation,
    })
}

/// A delegate which intercepts queries for the mana costs of cards while its
/// parent is face up and in play.
pub fn on_query_mana_cost(
    transformation: TransformationFn<CardId, Option<ManaValue>>,
) -> GameDelegate {
    GameDelegate::ManaCost(QueryDelegate {
        requirement: requirements::face_up_in_play,
        transformation,
    })
}

/// A delegate which intercepts queries for a card's [Resonance] set when a card
/// is face up & in play.
pub fn on_query_resonance(
    transformation: TransformationFn<CardId, EnumSet<Resonance>>,
) -> GameDelegate {
    delegates::on_query_resonance(requirements::face_up_in_play, transformation)
}

/// A delegate which intercepts queries for a player's maximum hand size while
/// its card is face up & in play.
pub fn on_query_maximum_hand_size(transformation: TransformationFn<Side, u32>) -> GameDelegate {
    delegates::maximum_hand_size(requirements::face_up_in_play, transformation)
}

/// A delegate which intercepts queries for a card's [CardStatusMarker]s when
/// its card is face up & in play.
pub fn on_query_card_status_markers(
    transformation: TransformationFn<CardId, Vec<CardStatusMarker>>,
) -> GameDelegate {
    delegates::on_query_card_status_markers(requirements::face_up_in_play, transformation)
}

/// A delegate which intercepts queries for whether an accessed card can be
/// scored while its card is face up & in play.
pub fn can_score_accessed_card(
    transformation: TransformationFn<AccessEvent<CardId>, Flag>,
) -> GameDelegate {
    delegates::can_score_accessed_card(requirements::face_up_in_play, transformation)
}

/// A delegate which intercepts queries for whether a player can currently win
/// the game by scoring points.
///
/// Note that if you prevent a player from winning via points, you are
/// responsible for checking for score victory if that effect ends by invoking
/// `mutations::check_for_score_victory()`
pub fn can_win_by_scoring_points(
    transformation: TransformationFn<Side, AbilityFlag>,
) -> GameDelegate {
    GameDelegate::CanWinGameViaPoints(QueryDelegate {
        requirement: requirements::face_up_in_play,
        transformation,
    })
}

/// A delegate which intercepts queries for whether the Covenant player can
/// currently score the given card
///
/// Note that if you prevent a scheme from being scored, you are responsible
/// for checking for schemes to score when that effect ends by invoking
/// `mutations::check_for_scoring_schemes()`.
pub fn can_covenant_score_scheme(
    transformation: TransformationFn<CardId, AbilityFlag>,
) -> GameDelegate {
    delegates::can_covenant_score_scheme(requirements::face_up_in_play, transformation)
}
