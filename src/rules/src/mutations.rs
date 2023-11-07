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

//! Core game mutations. In general, functions in this module append updates to
//! [GameState::animations].
//!
//! Mutation functions are expected to invoke a delegate event *after*
//! performing their mutation to inform other systems that game state
//! has changed.

use std::cmp;

use anyhow::Result;
use constants::game_constants;
use game_data::card_name::CardVariant;
use game_data::card_state::{CardCounter, CardState};
#[allow(unused)] // Used in rustdocs
use game_data::card_state::{CardData, CardPosition, CardPositionKind};
use game_data::delegate_data::{
    CardSacrificedEvent, DawnEvent, DiscardCardEvent, DiscardedCard, DiscardedFrom, DrawCardEvent,
    DuskEvent, EnterArenaEvent, MoveToDiscardPileEvent, OverlordScoreCardEvent, RaidEndEvent,
    RaidEvent, RaidFailureEvent, RaidOutcome, RaidSuccessEvent, ScoreCard, ScoreCardEvent,
    StoredManaTakenEvent, SummonMinionEvent, UnveilCardEvent,
};
use game_data::game_history::HistoryEvent;
use game_data::game_state::{GamePhase, GameState, RaidJumpRequest, TurnData, TurnState};
use game_data::game_updates::GameAnimation;
use game_data::primitives::{
    ActionCount, CardId, ManaValue, PointsValue, PowerChargeValue, RoomId, RoomLocation, Side,
    TurnNumber,
};
use game_data::{random, undo_tracker};
use tracing::{debug, instrument};
use with_error::{fail, verify};

use crate::mana::ManaPurpose;
use crate::{dispatch, flags, mana, queries, CardDefinitionExt};

/// Change a card to the 'face up' state and makes the card revealed to both
/// players.
pub fn turn_face_up(game: &mut GameState, card_id: CardId) {
    undo_tracker::track_revealed_state(game, card_id, |game| {
        game.card_mut(card_id).internal_turn_face_up()
    })
}

/// Change a card to the 'face down' state, but does *not* change its
/// revealed state for either player.
pub fn turn_face_down(game: &mut GameState, card_id: CardId) {
    game.card_mut(card_id).internal_turn_face_down();
}

/// Updates the 'revealed' state of a card to be visible to the indicated
/// `side` player. Note that this is *not* the same as turning a card
/// face-up, a card can be revealed to both players without being
/// face-up
pub fn set_revealed_to(game: &mut GameState, card_id: CardId, side: Side, revealed: bool) {
    undo_tracker::track_revealed_state(game, card_id, |game| {
        game.card_mut(card_id).internal_set_revealed_to(side, revealed);
    })
}

/// Move a card to a new position. Detects cases like drawing cards, playing
/// cards, and shuffling cards back into the deck and fires events
/// appropriately. The card will be placed in the position in global sorting-key
/// order, via [GameState::move_card_internal].
///
/// This function does *not* handle changing the 'revealed' or 'face down' state
/// of the card, the caller is responsible for updating that when the card moves
/// to a new game zone
pub fn move_card(game: &mut GameState, card_id: CardId, new_position: CardPosition) -> Result<()> {
    let name = game.card(card_id).variant;
    debug!(?name, ?card_id, ?new_position, "Moving card");
    let old_position = game.card(card_id).position();
    game.move_card_internal(card_id, new_position);

    if new_position.in_score_pile() {
        if queries::score(game, Side::Overlord) >= game_constants::POINTS_TO_WIN_GAME {
            game_over(game, Side::Overlord)?;
        }
        if queries::score(game, Side::Champion) >= game_constants::POINTS_TO_WIN_GAME {
            game_over(game, Side::Champion)?;
        }
    }

    if old_position.in_deck() && new_position.in_hand() {
        dispatch::invoke_event(game, DrawCardEvent(card_id))?;
    }

    if !old_position.in_play() && new_position.in_play() {
        game.card_mut(card_id).clear_arena_state();
        dispatch::invoke_event(game, EnterArenaEvent(card_id))?;
    }

    if new_position.in_discard_pile() {
        dispatch::invoke_event(game, MoveToDiscardPileEvent(card_id))?;
    }

    if old_position.in_deck() && new_position.in_discard_pile() {
        dispatch::invoke_event(
            game,
            DiscardCardEvent(DiscardedCard { card_id, discarded_from: DiscardedFrom::Deck }),
        )?;
    }

    if old_position.in_hand() && new_position.in_discard_pile() {
        dispatch::invoke_event(
            game,
            DiscardCardEvent(DiscardedCard { card_id, discarded_from: DiscardedFrom::Hand }),
        )?;
    }

    if new_position.in_discard_pile() && card_id.side == Side::Champion {
        turn_face_up(game, card_id);
    }

    Ok(())
}

/// Helper to move all cards in a list to a new [CardPosition] via [move_card].
pub fn move_cards(game: &mut GameState, cards: &[CardId], to_position: CardPosition) -> Result<()> {
    for card_id in cards {
        move_card(game, *card_id, to_position)?;
    }
    Ok(())
}

/// Move a card to the discard pile. This should specifically be used when an
/// opponent's effect uses the word 'destroy'.
pub fn destroy_card(game: &mut GameState, card_id: CardId) -> Result<()> {
    move_card(game, card_id, CardPosition::DiscardPile(card_id.side))?;
    if card_id.side == Side::Champion {
        turn_face_up(game, card_id);
    }
    Ok(())
}

/// Move a card to the discard pile. This should specifically be used when a
/// player's *own* effect causes their card to be discarded.
pub fn sacrifice_card(game: &mut GameState, card_id: CardId) -> Result<()> {
    move_card(game, card_id, CardPosition::DiscardPile(card_id.side))?;
    if card_id.side == Side::Champion {
        turn_face_up(game, card_id);
    }
    dispatch::invoke_event(game, CardSacrificedEvent(card_id))
}

/// Moves a card to the discard pile. This is precisely identical to calling
/// [move_card] for the discard pile position and only exists to improve
/// readability of code.
pub fn discard_card(game: &mut GameState, card_id: CardId) -> Result<()> {
    move_card(game, card_id, CardPosition::DiscardPile(card_id.side))
}

/// Shuffles the provided `cards` into the `side` player's deck, clearing their
/// revealed state for both players.
pub fn shuffle_into_deck(game: &mut GameState, side: Side, cards: &[CardId]) -> Result<()> {
    move_cards(game, cards, CardPosition::DeckUnknown(side))?;
    for card_id in cards {
        turn_face_down(game, *card_id);
        set_revealed_to(game, *card_id, Side::Overlord, false);
        set_revealed_to(game, *card_id, Side::Champion, false);
    }
    shuffle_deck(game, side)?;
    game.add_animation(|| GameAnimation::ShuffleIntoDeck);
    Ok(())
}

/// Shuffles the `side` player's deck, moving all cards into the `DeckUnknown`
/// card position.
pub fn shuffle_deck(game: &mut GameState, side: Side) -> Result<()> {
    let cards =
        game.cards_in_position(side, CardPosition::DeckTop(side)).map(|c| c.id).collect::<Vec<_>>();
    move_cards(game, &cards, CardPosition::DeckUnknown(side))
}

/// Helper function to draw `count` cards from the top of a player's deck and
/// place them into their hand.
///
/// If there are insufficient cards available:
///  - If `side == Overlord`, the Overlord player loses the game and no cards
///    are returned.
///  - If `side == Champion`, all remaining cards are returned.
///
/// Cards are marked as revealed to the `side` player. Returns a vector of the
/// newly-drawn [CardId]s.
pub fn draw_cards(game: &mut GameState, side: Side, count: u32) -> Result<Vec<CardId>> {
    let card_ids = realize_top_of_deck(game, side, count)?;

    if card_ids.len() != count as usize && side == Side::Overlord {
        game_over(game, side.opponent())?;
        return Ok(vec![]);
    }

    for card_id in &card_ids {
        set_revealed_to(game, *card_id, side, true);
    }

    game.add_animation(|| GameAnimation::DrawCards(side, card_ids.clone()));

    for card_id in &card_ids {
        move_card(game, *card_id, CardPosition::Hand(side))?;
    }

    Ok(card_ids)
}

/// Lose action points if a player has more than 0.
pub fn lose_action_points_if_able(
    game: &mut GameState,
    side: Side,
    amount: ActionCount,
) -> Result<()> {
    if game.player(side).actions > 0 {
        spend_action_points(game, side, amount)?;
    }
    Ok(())
}

/// Spends a player's action points.
///
/// Returns an error if sufficient action points are not available.
pub fn spend_action_points(game: &mut GameState, side: Side, amount: ActionCount) -> Result<()> {
    debug!(?side, ?amount, "Spending action points");
    verify!(game.player(side).actions >= amount, "Insufficient action points available");
    game.player_mut(side).actions -= amount;
    Ok(())
}

/// Adds action points for a player
pub fn gain_action_points(game: &mut GameState, side: Side, amount: ActionCount) -> Result<()> {
    debug!(?side, ?amount, "Gaining action points");
    game.player_mut(side).actions += amount;
    Ok(())
}

/// Adds bonus points to a player's score and checks for the Game Over
/// condition.
pub fn score_bonus_points(game: &mut GameState, side: Side, amount: PointsValue) -> Result<()> {
    game.player_mut(side).bonus_points += amount;
    if queries::score(game, side) >= game_constants::POINTS_TO_WIN_GAME {
        game_over(game, side)?;
    }
    Ok(())
}

/// Mark the game as won by the `winner` player.
pub fn game_over(game: &mut GameState, winner: Side) -> Result<()> {
    game.info.phase = GamePhase::GameOver { winner };
    game.add_animation(|| GameAnimation::GameOver(winner));
    Ok(())
}

/// Behavior when a card has no stored mana remaining after [take_stored_mana].
#[derive(Debug, Eq, PartialEq)]
pub enum OnZeroStored {
    Sacrifice,
    Ignore,
}

/// Add `amount` to the stored mana in a card. Returns the new stored amount.
pub fn add_stored_mana(game: &mut GameState, card_id: CardId, amount: ManaValue) -> ManaValue {
    game.card_mut(card_id).add_counters(CardCounter::StoredMana, amount);
    game.card(card_id).counters(CardCounter::StoredMana)
}

/// Takes *up to* `maximum` stored mana from a card and gives it to the player
/// who owns this card. Returns the amount of mana taken.
///
/// If no mana remains, the card is moved to its owner's discard pile if
/// `OnEmpty::MoveToDiscard` is specified.
pub fn take_stored_mana(
    game: &mut GameState,
    card_id: CardId,
    maximum: ManaValue,
    on_zero_stored: OnZeroStored,
) -> Result<ManaValue> {
    debug!(?card_id, ?maximum, "Taking stored mana");
    let available = game.card(card_id).counters(CardCounter::StoredMana);
    let taken = cmp::min(available, maximum);
    game.card_mut(card_id).remove_counters_saturating(CardCounter::StoredMana, taken);
    mana::gain(game, card_id.side, taken);
    dispatch::invoke_event(game, StoredManaTakenEvent(card_id))?;

    if on_zero_stored == OnZeroStored::Sacrifice
        && game.card(card_id).counters(CardCounter::StoredMana) == 0
    {
        sacrifice_card(game, card_id)?;
    }

    Ok(taken)
}

/// Adds `count` power charges to the `card_id` card.
pub fn add_power_charges(
    game: &mut GameState,
    card_id: CardId,
    count: PowerChargeValue,
) -> Result<()> {
    game.card_mut(card_id).add_counters(CardCounter::PowerCharges, count);
    Ok(())
}

/// Spends `count` power charges from the `card_id` card.
///
/// Returns an error if insufficient charges are available.
pub fn spend_power_charges(
    game: &mut GameState,
    card_id: CardId,
    count: PowerChargeValue,
) -> Result<()> {
    let card = game.card_mut(card_id);
    verify!(
        card.counters(CardCounter::PowerCharges) >= count,
        "Insufficient power charges available"
    );
    card.remove_counters_saturating(CardCounter::PowerCharges, count);
    Ok(())
}

/// Ends the current raid. Returns an error if no raid is currently active.
pub fn end_raid(game: &mut GameState, outcome: RaidOutcome) -> Result<()> {
    debug!("Ending raid");
    let target = game.raid()?.target;
    let event = RaidEvent { raid_id: game.raid()?.raid_id, target, data: () };
    match outcome {
        RaidOutcome::Success => {
            dispatch::invoke_event(game, RaidSuccessEvent(event))?;
            game.add_history_event(HistoryEvent::RaidSuccess(event));
        }
        RaidOutcome::Failure => {
            dispatch::invoke_event(game, RaidFailureEvent(event))?;
            game.add_history_event(HistoryEvent::RaidFailure(event));
        }
    }
    dispatch::invoke_event(
        game,
        RaidEndEvent(RaidEvent { raid_id: event.raid_id, target: event.target, data: outcome }),
    )?;
    game.raid = None;
    Ok(())
}

/// Deals initial hands to both players and prompts for mulligan decisions.
#[instrument(skip(game))]
pub fn deal_opening_hands(game: &mut GameState) -> Result<()> {
    debug!("Dealing opening hands");
    draw_cards(game, Side::Overlord, game_constants::STARTING_HAND_SIZE)?;
    draw_cards(game, Side::Champion, game_constants::STARTING_HAND_SIZE)?;
    Ok(())
}

/// Invoked after a mulligan decision is received in order to check if the game
/// should be started.
///
/// Handles assigning initial mana & action points to players.
#[instrument(skip(game))]
pub fn check_start_game(game: &mut GameState) -> Result<()> {
    match &game.info.phase {
        GamePhase::ResolveMulligans(mulligans)
            if mulligans.overlord.is_some() && mulligans.champion.is_some() =>
        {
            mana::set(game, Side::Overlord, game_constants::STARTING_MANA);
            mana::set(game, Side::Champion, game_constants::STARTING_MANA);
            start_turn(game, Side::Overlord, 1)?;
        }
        _ => {}
    }
    Ok(())
}

/// Returns a list of *up to* `count` cards from the top of the `side` player's
/// deck, in sorting-key order (later indices are are closer to the top
/// of the deck).
///
/// Selects randomly unless cards are already known to be in this position.
/// If insufficient cards are present in the deck, returns all available
/// cards. Cards are moved to the DeckTop position via [move_card],
/// meaning that subsequent calls to this function will see the same results.
///
/// Does not change the 'revealed' state of cards.
pub fn realize_top_of_deck(game: &mut GameState, side: Side, count: u32) -> Result<Vec<CardId>> {
    let count = count as usize; // don't run this on 16 bit processors please :)
    let mut cards = game.card_list_for_position(side, CardPosition::DeckTop(side));
    let len = cards.len();
    let result = if count <= len {
        cards[(len - count)..len].to_vec()
    } else {
        let remaining = count - cards.len();
        let mut shuffled =
            random::cards_in_position(game, side, CardPosition::DeckUnknown(side), remaining);
        shuffled.append(&mut cards);
        shuffled
    };

    for card_id in &result {
        move_card(game, *card_id, CardPosition::DeckTop(side))?;
    }

    Ok(result)
}

/// Increases the progress level of all `can_progress_card` Overlord cards in a
/// room by one. If a Scheme card's progress level reaches its
/// `progress_requirement`, that card is immediately scored and moved to the
/// Overlord score zone.
///
/// Does not spend mana/actions etc.
pub fn progress_room(game: &mut GameState, room_id: RoomId) -> Result<()> {
    let occupants = game.card_list_for_position(
        Side::Overlord,
        CardPosition::Room(room_id, RoomLocation::Occupant),
    );
    let can_progress = occupants
        .into_iter()
        .filter(|card_id| flags::can_progress_card(game, *card_id))
        .collect::<Vec<_>>();

    for occupant_id in can_progress {
        add_progress_counters(game, occupant_id, 1)?;
    }

    Ok(())
}

/// Adds `amount` progress counters to the provided card.
///
/// If the card has scheme points and the progress requirement is met, the card
/// is immediately scored and moved to the Overlord's score zone.
///
/// Returns an error if this card cannot be progressed.
pub fn add_progress_counters(game: &mut GameState, card_id: CardId, amount: u32) -> Result<()> {
    verify!(flags::can_progress_card(game, card_id));
    game.card_mut(card_id).add_counters(CardCounter::Progress, amount);
    let card = game.card(card_id);
    if let Some(scheme_points) = crate::get(card.variant).config.stats.scheme_points {
        if card.counters(CardCounter::Progress) >= scheme_points.progress_requirement {
            turn_face_up(game, card_id);
            move_card(game, card_id, CardPosition::Scoring)?;
            game.add_animation(|| GameAnimation::ScoreCard(Side::Overlord, card_id));
            dispatch::invoke_event(game, OverlordScoreCardEvent(card_id))?;
            dispatch::invoke_event(
                game,
                ScoreCardEvent(ScoreCard { player: Side::Overlord, card_id }),
            )?;

            move_card(game, card_id, CardPosition::Scored(Side::Overlord))?;
        }
    }

    Ok(())
}

/// Pays a card's cost and turns it face up, returning an error if the
/// card is already face-up or cannot be unveiled for any other reason.
pub fn unveil_card(game: &mut GameState, card_id: CardId) -> Result<()> {
    verify!(game.card(card_id).is_face_down(), "Card is not face-down");
    verify!(game.card(card_id).position().in_play(), "Card is not in play");

    if let Some(custom_cost) = &game.card(card_id).definition().cost.custom_cost {
        if (custom_cost.can_pay)(game, card_id) {
            (custom_cost.pay)(game, card_id)?;
        } else {
            fail!("Cannot pay custom cost for project");
        }
    }

    match queries::mana_cost(game, card_id) {
        None => {
            turn_face_up(game, card_id);
        }
        Some(cost) if cost <= mana::get(game, card_id.side, ManaPurpose::PayForCard(card_id)) => {
            mana::spend(game, card_id.side, ManaPurpose::PayForCard(card_id), cost)?;
            turn_face_up(game, card_id);
        }
        _ => fail!("Insufficient mana available to unveil project"),
    }

    game.add_animation(|| GameAnimation::UnveilCard(card_id));
    dispatch::invoke_event(game, UnveilCardEvent(card_id))?;

    Ok(())
}

/// Equivalent function to [unveil_card] which ignores costs.
pub fn unveil_card_ignoring_costs(game: &mut GameState, card_id: CardId) -> Result<()> {
    verify!(game.card(card_id).is_face_down(), "Card is not face-down");
    verify!(game.card(card_id).position().in_play(), "Card is not in play");

    turn_face_up(game, card_id);
    game.add_animation(|| GameAnimation::UnveilCard(card_id));
    dispatch::invoke_event(game, UnveilCardEvent(card_id))?;

    Ok(())
}

/// Starts the turn for the `next_side` player.
pub fn start_turn(game: &mut GameState, next_side: Side, turn_number: TurnNumber) -> Result<()> {
    game.info.phase = GamePhase::Play;
    game.info.turn = TurnData { side: next_side, turn_number };
    game.info.turn_state = TurnState::Active;

    debug!(?next_side, "Starting player turn");
    game.add_animation(|| GameAnimation::StartTurn(next_side));

    if next_side == Side::Overlord {
        dispatch::invoke_event(game, DuskEvent(turn_number))?;
    } else {
        dispatch::invoke_event(game, DawnEvent(turn_number))?;
    }
    game.player_mut(next_side).actions = queries::start_of_turn_action_count(game, next_side);

    if next_side == Side::Overlord {
        draw_cards(game, next_side, 1)?;
    }

    Ok(())
}

/// Options when invoking [summon_minion]
#[derive(Eq, PartialEq, Debug)]
pub enum SummonMinion {
    PayCosts,
    IgnoreCosts,
}

/// Turn a minion card in play face up, paying its costs based on the
/// [SummonMinion] value provided.
///
/// Returns an error if the indicated card is already face-up.
pub fn summon_minion(game: &mut GameState, card_id: CardId, costs: SummonMinion) -> Result<()> {
    verify!(game.card(card_id).is_face_down());
    if costs == SummonMinion::PayCosts {
        if let Some(cost) = queries::mana_cost(game, card_id) {
            mana::spend(game, Side::Overlord, ManaPurpose::PayForCard(card_id), cost)?;
        }

        if let Some(custom_cost) = &game.card(card_id).definition().cost.custom_cost {
            (custom_cost.pay)(game, card_id)?;
        }
    }

    dispatch::invoke_event(game, SummonMinionEvent(card_id))?;
    turn_face_up(game, card_id);
    game.add_animation(|| GameAnimation::SummonMinion(card_id));
    Ok(())
}

/// Turn a minion card in play face down, if able.
pub fn unsummon_minion(game: &mut GameState, card_id: CardId) -> Result<()> {
    turn_face_down(game, card_id);
    game.add_animation(|| GameAnimation::UnsummonMinion(card_id));
    Ok(())
}

/// Discards `amount` cards from the top of the Overlord's deck.
///
/// If insufficient cards are present, discards all available cards.
pub fn discard_from_vault(game: &mut GameState, amount: u32) -> Result<()> {
    for card_id in realize_top_of_deck(game, Side::Overlord, amount)? {
        move_card(game, card_id, CardPosition::DiscardPile(Side::Overlord))?;
    }
    Ok(())
}

/// Stops the currently-active 'play card' game action.
pub fn abort_playing_card(game: &mut GameState) {
    game.state_machines.play_card = None;
}

/// Applies a [RaidJumpRequest] to the provided `game` if there is currently an
/// active raid.
pub fn apply_raid_jump(game: &mut GameState, request: RaidJumpRequest) {
    if let Some(raid) = game.raid.as_mut() {
        raid.jump_request = Some(request);
    }
}

/// Creates an entirely new card from outside the game face-up in the indicated
/// `position`.
pub fn create_and_add_card(
    game: &mut GameState,
    variant: CardVariant,
    position: CardPosition,
) -> Result<()> {
    let definition = crate::get(variant);
    let side = definition.side;
    let card_id = CardId::new(side, game.cards(side).len());
    let state = CardState::new_with_position(
        card_id,
        variant,
        position,
        game.next_sorting_key(),
        true, /* is_face_up */
    );
    game.cards_mut(side).push(state);
    dispatch::add_card_to_delegate_cache(
        &mut game.delegate_cache,
        definition,
        card_id,
        variant.metadata,
    );
    debug!(?variant, ?card_id, ?position, "Created new external card");
    Ok(())
}

/// Overwrites an existing card with a completely new card from outside the
/// game, face-down in the same position as the current card. All existing card
/// state is discarded.
pub fn overwrite_card(game: &mut GameState, card_id: CardId, new: CardVariant) -> Result<()> {
    let old_definition = crate::get(game.card(card_id).variant);
    let position = game.card(card_id).position();
    let sorting_key = game.card(card_id).sorting_key;
    *game.card_mut(card_id) =
        CardState::new_with_position(card_id, new, position, sorting_key, false);
    dispatch::remove_card_from_delegate_cache(&mut game.delegate_cache, old_definition, card_id);
    let new_definition = crate::get(new);
    dispatch::add_card_to_delegate_cache(
        &mut game.delegate_cache,
        new_definition,
        card_id,
        new.metadata,
    );
    debug!(?new, ?card_id, "Overwrote existing card with new card");
    Ok(())
}
