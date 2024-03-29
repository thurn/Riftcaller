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

//! Functions to query boolean game information, typically whether some game
//! action can currently be taken

use card_definition_data::ability_data::AbilityType;
use card_definition_data::cards;
use cards::CardDefinitionExt;
use constants::game_constants;
use core_data::game_primitives::{
    AbilityId, CardId, CardSubtype, CardType, RaidId, RoomId, RoomLocation, Side,
};
use dispatcher::dispatch;
use game_data::card_configuration::TargetRequirement;
use game_data::card_state::CardPosition;
use game_data::delegate_data::{
    CanActivateAbility, CanActivateAbilityQuery, CanCovenantScoreSchemeQuery,
    CanEndRaidAccessPhaseQuery, CanEvadeMinionQuery, CanInitiateRaidQuery,
    CanMinionBeDefeatedQuery, CanPlayCardQuery, CanProgressCardQuery, CanProgressRoomQuery,
    CanSummonQuery, CanTakeDrawCardActionQuery, CanTakeGainManaActionQuery, CanUseNoWeaponQuery,
    CanWinGameViaPointsQuery,
};
use game_data::flag_data::{AbilityFlag, Flag};
use game_data::game_actions::CardTarget;
use game_data::game_state::{GamePhase, GameState, TurnState};
use game_data::prompt_data::{
    CardSelectorPrompt, CardSelectorPromptValidation, GamePrompt, PlayCardBrowser,
};
use game_data::raid_data::RaidStatus;
use game_data::state_machine_data::PlayCardOptions;
use game_data::utils;

use crate::mana::ManaPurpose;
use crate::{curses, mana, prompts, queries};

/// Returns the player that is currently able to take actions in the provided
/// game. If no player can act, e.g. because the game has ended, returns None.
pub fn current_priority(game: &GameState) -> Option<Side> {
    match &game.info.phase {
        GamePhase::ResolveMulligans(_) => {
            if can_make_mulligan_decision(game, Side::Covenant) {
                Some(Side::Covenant)
            } else if can_make_mulligan_decision(game, Side::Riftcaller) {
                Some(Side::Riftcaller)
            } else {
                None
            }
        }
        GamePhase::Play => {
            if !prompts::is_empty(game, Side::Covenant) {
                Some(Side::Covenant)
            } else if !prompts::is_empty(game, Side::Riftcaller) {
                Some(Side::Riftcaller)
            } else if let Some(raid) = &game.raid {
                Some(match queries::raid_status(raid) {
                    RaidStatus::Begin | RaidStatus::Encounter | RaidStatus::Access => {
                        Side::Riftcaller
                    }
                    RaidStatus::Summon | RaidStatus::ApproachRoom => Side::Covenant,
                })
            } else if game.info.turn_state == TurnState::Active {
                Some(game.info.turn.side)
            } else {
                Some(game.info.turn.side.opponent())
            }
        }
        GamePhase::GameOver { .. } => None,
    }
}

/// Returns true if the `side` player currently has priority as described by
/// [current_priority].
pub fn has_priority(game: &GameState, side: Side) -> bool {
    current_priority(game) == Some(side)
}

/// Returns whether a player can currently make a mulligan decision
pub fn can_make_mulligan_decision(game: &GameState, side: Side) -> bool {
    match &game.info.phase {
        GamePhase::ResolveMulligans(mulligans) => {
            if mulligans.decision(side).is_none() {
                match side {
                    // Covenant resolves mulligans first
                    Side::Covenant => true,
                    Side::Riftcaller => mulligans.decision(Side::Covenant).is_some(),
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Returns true if the owner of the `card_id` card can currently pay its cost.
///
/// See [can_play_card] for a function which checks all factor related to
/// playing a card.
pub fn can_pay_card_cost(game: &GameState, card_id: CardId, options: PlayCardOptions) -> bool {
    let mut can_pay = if options.ignore_mana_cost {
        true
    } else {
        matches!(queries::mana_cost(game, card_id), Some(cost)
                             if cost <= mana::get(game, card_id.side, ManaPurpose::PayForCard(card_id)))
    };
    if let Some(custom_cost) = &game.card(card_id).definition().cost.custom_cost {
        can_pay &= (custom_cost.can_pay)(game, card_id);
    }

    can_pay
}

/// Returns whether a given card can currently be played.
pub fn can_play_card(
    game: &GameState,
    side: Side,
    card_id: CardId,
    target: CardTarget,
    options: PlayCardOptions,
) -> bool {
    let query: bool =
        dispatch::perform_query(game, CanPlayCardQuery(&card_id), Flag::new(true)).into();
    if !query {
        return false;
    }

    if let Some(GamePrompt::PlayCardBrowser(browser)) = &prompts::current(game, card_id.side) {
        return can_play_from_browser(game, card_id, target, browser);
    }

    let mut can_play = (in_main_phase_with_action_point(game, side) || options.ignore_phase)
        && side == card_id.side
        && (game.card(card_id).position() == CardPosition::Hand(side) || options.ignore_position)
        && is_valid_target(game, card_id, target)
        && (options.ignore_action_cost
            || queries::action_cost(game, card_id) <= game.player(side).actions);

    if enters_play_face_up(game, card_id) {
        can_play &= can_pay_card_cost(game, card_id, options);
    }

    can_play
}

/// Checks whether a card can be played from a [PlayCardBrowser].
///
/// Cards in the browser are assumed to bypass normal positional checks for
/// legality.
fn can_play_from_browser(
    game: &GameState,
    card_id: CardId,
    target: CardTarget,
    browser: &PlayCardBrowser,
) -> bool {
    let mut can_play = browser.cards.contains(&card_id)
        && is_valid_target(game, card_id, target)
        && queries::action_cost(game, card_id) <= game.player(card_id.side).actions;

    if enters_play_face_up(game, card_id) {
        can_play &= can_pay_card_cost(game, card_id, PlayCardOptions::default());
    }

    dispatch::perform_query(game, CanPlayCardQuery(&card_id), Flag::new(can_play)).into()
}

/// Whether the `ability_id` ability can currently be activated with the
/// provided `target`.
pub fn can_take_activate_ability_action(
    game: &GameState,
    side: Side,
    ability_id: AbilityId,
    target: CardTarget,
) -> bool {
    let card = game.card(ability_id.card_id);

    let AbilityType::Activated { cost, target_requirement } =
        &cards::ability_definition(game, ability_id).ability_type
    else {
        return false;
    };

    if !matching_targeting(game, target_requirement, ability_id, target) {
        return false;
    }

    let mut can_activate = can_take_game_actions(game, side)
        && side == ability_id.side()
        && has_priority(game, side)
        && card.is_face_up()
        && card.position().in_play()
        // Abilities with an action point cost cannot be activated at instant
        // speed
        && (cost.actions == 0 || in_main_phase_with_action_point(game, side));

    if side == Side::Covenant && cost.actions == 0 {
        // Covenant abilities with no action point cost can only be activated
        // when their activation window is open, as determined by their
        // subtypes.
        can_activate &= can_activate_for_subtypes(game, ability_id.card_id)
    }

    if let Some(custom_cost) = &cost.custom_cost {
        can_activate &= (custom_cost.can_pay)(game, ability_id);
    }

    can_activate &= cost.actions <= game.player(side).actions;

    if let Some(cost) = queries::ability_mana_cost(game, ability_id) {
        can_activate &= cost <= mana::get(game, side, ManaPurpose::ActivateAbility(ability_id));
    }

    dispatch::perform_query(
        game,
        CanActivateAbilityQuery(&CanActivateAbility { ability_id, target }),
        Flag::new(can_activate),
    )
    .into()
}

/// Returns true if the `ability_id` ability could be activated with a valid
/// target.
pub fn activated_ability_has_valid_targets(
    game: &GameState,
    side: Side,
    ability_id: AbilityId,
) -> bool {
    match &cards::ability_definition(game, ability_id).ability_type {
        AbilityType::Activated { target_requirement, .. } => match target_requirement {
            TargetRequirement::None => {
                can_take_activate_ability_action(game, side, ability_id, CardTarget::None)
            }
            TargetRequirement::TargetRoom(_) => enum_iterator::all::<RoomId>().any(|room_id| {
                can_take_activate_ability_action(game, side, ability_id, CardTarget::Room(room_id))
            }),
        },
        _ => false,
    }
}

/// Returns true if the provided card has any activated ability that can
/// currently be used.
fn can_use_any_card_ability(game: &GameState, card_id: CardId) -> bool {
    let definition = game.card(card_id).definition();
    definition
        .ability_ids(card_id)
        .any(|ability_id| activated_ability_has_valid_targets(game, card_id.side, ability_id))
}

fn is_valid_target(game: &GameState, card_id: CardId, target: CardTarget) -> bool {
    let definition = cards::get(game.card(card_id).variant);
    if let Some(targeting) = &definition.config.custom_targeting {
        return matching_targeting(game, targeting, card_id, target);
    }

    match definition.card_type {
        CardType::Spell
        | CardType::Artifact
        | CardType::Evocation
        | CardType::Ally
        | CardType::Ritual => target == CardTarget::None,
        CardType::Minion => matches!(target, CardTarget::Room(_)),
        CardType::Project | CardType::Scheme => {
            matches!(target, CardTarget::Room(room_id) if room_id.is_outer_room())
        }
        CardType::GameModifier | CardType::Riftcaller | CardType::Chapter | CardType::Sigil => {
            false
        }
    }
}

/// Returns true if the targeting requirement in `requirement` matches the
/// target in `target`.
fn matching_targeting<T>(
    game: &GameState,
    requirement: &TargetRequirement<T>,
    data: T,
    target: CardTarget,
) -> bool {
    match (requirement, target) {
        (TargetRequirement::None, CardTarget::None) => true,
        (TargetRequirement::TargetRoom(predicate), CardTarget::Room(room_id)) => {
            predicate(game, data, room_id)
        }
        _ => false,
    }
}

/// Returns true if the indicated card should enter play in the face up state
/// and is expected to pay its costs immediately.
pub fn enters_play_face_up(game: &GameState, card_id: CardId) -> bool {
    !matches!(
        cards::get(game.card(card_id).variant).card_type,
        CardType::Minion | CardType::Scheme | CardType::Project
    )
}

/// Returns true if the indicated card should enter play in a room
pub fn enters_play_in_room(game: &GameState, card_id: CardId) -> bool {
    matches!(
        cards::get(game.card(card_id).variant).card_type,
        CardType::Minion | CardType::Scheme | CardType::Project
    )
}

/// Returns whether the indicated player can currently take the basic game
/// action to draw a card.
pub fn can_take_draw_card_action(game: &GameState, side: Side) -> bool {
    let can_draw = in_main_phase_with_action_point(game, side) && game.deck(side).next().is_some();
    dispatch::perform_query(game, CanTakeDrawCardActionQuery(&side), Flag::new(can_draw)).into()
}

/// Returns whether the indicated player can currently take the basic game
/// action to gain one mana.
pub fn can_take_gain_mana_action(game: &GameState, side: Side) -> bool {
    let can_gain_mana = in_main_phase_with_action_point(game, side);
    dispatch::perform_query(game, CanTakeGainManaActionQuery(&side), Flag::new(can_gain_mana))
        .into()
}

/// Returns whether the indicated room could be accessed by an ability based on
/// its properties (e.g. whether it is occupied).
pub fn is_valid_access_target(game: &GameState, room_id: RoomId) -> bool {
    room_id.is_inner_room() || game.occupants(room_id).next().is_some()
}

/// Returns whether the indicated room could be the target of a raid based on
/// its properties (occupancy, etc). To check whether the 'initiate raid' action
/// itself can be performed, call [can_take_initiate_raid_action] instead.
pub fn is_valid_raid_target(game: &GameState, room_id: RoomId) -> bool {
    dispatch::perform_query(
        game,
        CanInitiateRaidQuery(&room_id),
        Flag::new(is_valid_access_target(game, room_id)),
    )
    .into()
}

/// Returns whether a raid is currently active in this game.
pub fn raid_active(game: &GameState) -> bool {
    let Some(raid) = &game.raid else {
        return false;
    };

    !raid.is_custom_access
}

/// Returns whether the indicated player can currently take the basic game
/// action to initiate a raid on the target [RoomId].
pub fn can_take_initiate_raid_action(game: &GameState, side: Side, room_id: RoomId) -> bool {
    side == Side::Riftcaller
        && game.raid.is_none()
        && in_main_phase_with_action_point(game, side)
        && is_valid_raid_target(game, room_id)
}

/// Returns whether the indicated player can currently take the basic game
/// action to progress a room.
pub fn can_take_progress_action(game: &GameState, side: Side, room_id: RoomId) -> bool {
    let has_progress_card = game
        .occupants(room_id)
        .chain(game.defenders_unordered(room_id))
        .any(|card| can_progress_card(game, card.id));
    let can_progress = has_progress_card
        && side == Side::Covenant
        && mana::get(game, side, ManaPurpose::ProgressRoom(room_id)) > 0
        && in_main_phase_with_action_point(game, side);
    dispatch::perform_query(game, CanProgressRoomQuery(&room_id), Flag::new(can_progress)).into()
}

/// Whether the indicated card can be progressed when the 'progress' action
/// is taken for its room or via a card ability.
pub fn can_progress_card(game: &GameState, card_id: CardId) -> bool {
    let can_progress = game.card(card_id).definition().card_type == CardType::Scheme;
    dispatch::perform_query(game, CanProgressCardQuery(&card_id), Flag::new(can_progress)).into()
}

/// Whether the indicated player can currently take any type of game state
/// actions.
pub fn can_take_game_state_actions(game: &GameState, user_side: Side) -> bool {
    prompts::is_empty(game, user_side) && current_priority(game) == Some(user_side)
}

/// Returns true if the `side` player is in their main phase as described in
/// [in_main_phase] and they have more than zero action points available.
pub fn in_main_phase_with_action_point(game: &GameState, side: Side) -> bool {
    in_main_phase(game, side) && game.player(side).actions > 0
}

/// Returns true if the provided `side` player is currently in their Main phase
/// with no pending prompt responses, and thus can take a primary game action.
pub fn in_main_phase(game: &GameState, side: Side) -> bool {
    can_take_game_actions(game, side)
        && game.info.turn.side == side
        && game.info.turn_state != TurnState::Ended
        && game.raid.is_none()
}

/// Returns true if the `side` player's prompt queue is current empty *or* if
/// their current prompt is a [GamePrompt::PriorityPrompt].
pub fn prompt_queue_empty_or_has_priority_prompt(game: &GameState, side: Side) -> bool {
    prompts::is_empty(game, side)
        || matches!(prompts::current(game, side), Some(GamePrompt::PriorityPrompt))
}

/// Returns true if either player can currently take game standard game actions
/// This generally means the game is currently in progress and neither player is
/// facing a card prompt.
pub fn can_take_game_actions(game: &GameState, side: Side) -> bool {
    game.info.phase.is_playing()
        && prompt_queue_empty_or_has_priority_prompt(game, side)
        && prompts::is_empty(game, side.opponent())
}

/// Returns true if the `card_id` is currently face down and could be
/// turned face up by paying its mana cost.
///
/// Returns an error if there is no active raid or if this is an invalid
/// defender.
pub fn can_summon(game: &GameState, card_id: CardId) -> bool {
    let can_summon = game.card(card_id).is_face_down()
        && can_pay_card_cost(game, card_id, PlayCardOptions::default());
    dispatch::perform_query(game, CanSummonQuery(&card_id), Flag::new(can_summon)).into()
}

/// Can the Riftcaller choose to not use a weapon ability when encountering
/// the indicated minion card?
pub fn can_take_use_no_weapon_action(game: &GameState, card_id: CardId) -> bool {
    dispatch::perform_query(
        game,
        CanUseNoWeaponQuery(&card_id),
        Flag::new(can_take_game_actions(game, Side::Riftcaller)),
    )
    .into()
}

/// Can the minion currently being encountered be evaded?
pub fn can_evade_current_minion(game: &GameState) -> bool {
    let Some(minion_id) = game.current_raid_defender() else {
        return false;
    };
    dispatch::perform_query(game, CanEvadeMinionQuery(&minion_id), Flag::new(true)).into()
}

/// Can the minion currently being encountered be defeated at all?
///
/// This only describes whether the minion can be defeated by any means, not
/// whether the Riftcaller player is currently able to do so.
pub fn can_defeat_current_minion(game: &GameState) -> bool {
    let Some(minion_id) = game.current_raid_defender() else {
        return false;
    };
    dispatch::perform_query(game, CanMinionBeDefeatedQuery(&minion_id), Flag::new(true)).into()
}

/// Returns true if the [CardId] card is currently the outermost defender in a
/// room.
pub fn is_outermost_defender(game: &GameState, card_id: CardId) -> bool {
    if let CardPosition::Room(_, room_id, RoomLocation::Defender) = game.card(card_id).position() {
        game.defender_list(room_id).last() == Some(&card_id)
    } else {
        false
    }
}

/// Can the Riftcaller choose to use the 'End Raid' button to end the access
/// phase of a raid?
pub fn can_take_end_raid_access_phase_action(game: &GameState, raid_id: RaidId) -> bool {
    dispatch::perform_query(
        game,
        CanEndRaidAccessPhaseQuery(&raid_id),
        Flag::new(can_take_game_actions(game, Side::Riftcaller)),
    )
    .into()
}

/// Returns whether a player can currently take the 'end turn' action.
pub fn can_take_end_turn_action(game: &GameState, side: Side) -> bool {
    in_main_phase(game, side) && game.player(side).actions == 0
}

/// Returns whether a player can currently take the 'start turn' action.
pub fn can_take_start_turn_action(game: &GameState, side: Side) -> bool {
    can_take_game_actions(game, side)
        && game.info.turn.side == side.opponent()
        && game.info.turn_state == TurnState::Ended
}

/// Is the `side` player currently able to summon the provided project?
pub fn can_take_summon_project_action(game: &GameState, side: Side, card_id: CardId) -> bool {
    let definition = &game.card(card_id).definition();
    can_take_game_actions(game, side)
        && side == Side::Covenant
        && has_priority(game, side)
        && game.card(card_id).is_face_down()
        && game.card(card_id).position().in_play()
        && definition.card_type == CardType::Project
        && can_activate_for_subtypes(game, card_id)
        && can_summon(game, card_id)
}

/// Can the `side` player currently take the standard action to remove a curse?
pub fn can_take_remove_curse_action(game: &GameState, side: Side) -> bool {
    side == Side::Riftcaller
        && in_main_phase_with_action_point(game, side)
        && mana::get(game, side, ManaPurpose::RemoveCurse) >= game_constants::COST_TO_REMOVE_CURSE
}

/// Can the `side` player currently take the standard action to dispel an
/// evocation?
pub fn can_take_dispel_evocation_action(game: &GameState, side: Side) -> bool {
    side == Side::Covenant
        && in_main_phase_with_action_point(game, side)
        && curses::get(game) > 0
        && game.evocations().count() > 0
        && mana::get(game, side, ManaPurpose::DispelEvocation)
            >= game_constants::COST_TO_DISPEL_EVOCATION
}

/// Returns true if the Covenant player currently has access to an effect they
/// can activate outside of their normal main phase actions.
pub fn covenant_has_instant_speed_actions(game: &GameState) -> bool {
    game.occupants_in_all_rooms()
        .any(|c| can_take_summon_project_action(game, Side::Covenant, c.id))
        || game.all_permanents(Side::Covenant).any(|c| can_use_any_card_ability(game, c.id))
}

/// Checks whether a Covenant card is currently in its assigned activation
/// window based on its subtypes and can thus be summoned or activated.
///
/// Does not check legality of activation beyond the card's subtypes.
pub fn can_activate_for_subtypes(game: &GameState, card_id: CardId) -> bool {
    let subtypes = &game.card(card_id).definition().subtypes;
    let current_turn = game.info.turn.side;
    let turn_state = game.info.turn_state;

    let duskbound = subtypes.contains(&CardSubtype::Duskbound)
        && current_turn == Side::Riftcaller
        && turn_state == TurnState::Ended;
    let nightbound = subtypes.contains(&CardSubtype::Nightbound)
        && current_turn == Side::Covenant
        && turn_state != TurnState::Ended;
    let summonbound = subtypes.contains(&CardSubtype::Summonbound)
        && current_turn == Side::Riftcaller
        && utils::is_true(|| Some(queries::raid_status(game.raid.as_ref()?) == RaidStatus::Summon));
    let roombound = subtypes.contains(&CardSubtype::Roombound)
        && current_turn == Side::Riftcaller
        && utils::is_true(|| {
            Some(queries::raid_status(game.raid.as_ref()?) == RaidStatus::ApproachRoom)
        });

    duskbound || nightbound || summonbound || roombound
}

/// Returns true if the validation logic for this [CardSelectorPrompt] is
/// currently satisfied.
pub fn card_selector_state_is_valid(prompt: &CardSelectorPrompt) -> bool {
    match prompt.validation {
        Some(CardSelectorPromptValidation::ExactlyCount(count)) => {
            prompt.chosen_subjects.len() == count
        }
        Some(CardSelectorPromptValidation::LessThanOrEqualTo(count)) => {
            prompt.chosen_subjects.len() <= count
        }
        Some(CardSelectorPromptValidation::AllSubjects) => prompt.unchosen_subjects.is_empty(),
        None => true,
    }
}

/// Can the [Side] player currently win the game by scoring points?
pub fn can_win_game_via_points(game: &GameState, side: Side) -> AbilityFlag {
    dispatch::perform_query(game, CanWinGameViaPointsQuery(&side), AbilityFlag::new(true))
}

/// Can the Covenant player currently score the [CardId] scheme?
pub fn can_covenant_score_scheme(game: &GameState, card_id: CardId) -> AbilityFlag {
    dispatch::perform_query(game, CanCovenantScoreSchemeQuery(&card_id), AbilityFlag::new(true))
}
