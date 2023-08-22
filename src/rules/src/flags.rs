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

//! Functions to query boolean game information, typically whether some game
//! action can currently be taken

use game_data::card_definition::{AbilityType, TargetRequirement};
use game_data::card_state::CardPosition;
use game_data::delegates::{
    CanActivateAbilityQuery, CanDefeatTargetQuery, CanEncounterTargetQuery,
    CanEndRaidAccessPhaseQuery, CanInitiateRaidQuery, CanLevelUpCardQuery, CanLevelUpRoomQuery,
    CanPlayCardQuery, CanTakeDrawCardActionQuery, CanTakeGainManaActionQuery, CanUseNoWeaponQuery,
    CardEncounter, Flag,
};
use game_data::game::{GamePhase, GameState, TurnState};
use game_data::game_actions::CardTarget;
use game_data::primitives::{AbilityId, CardId, CardType, Lineage, RaidId, RoomId, Side};

use crate::mana::ManaPurpose;
use crate::{dispatch, mana, queries};

/// Returns whether a player can currently make a mulligan decision
pub fn can_make_mulligan_decision(game: &GameState, side: Side) -> bool {
    match &game.info.phase {
        GamePhase::ResolveMulligans(mulligans) => {
            if mulligans.decision(side).is_none() {
                match side {
                    // Overlord resolves mulligans first
                    Side::Overlord => true,
                    Side::Champion => mulligans.decision(Side::Overlord).is_some(),
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
/// See [can_take_play_card_action] for a function which checks all factors
/// related to playing a card.
fn can_pay_card_cost(game: &GameState, card_id: CardId) -> bool {
    let mut can_pay = matches!(queries::mana_cost(game, card_id), Some(cost)
                             if cost <= mana::get(game, card_id.side, ManaPurpose::PayForCard(card_id)));
    if let Some(custom_cost) = &crate::card_definition(game, card_id).cost.custom_cost {
        can_pay &= (custom_cost.can_pay)(game, card_id);
    }

    can_pay
}

/// Returns whether a given card can currently be played via the basic game
/// action to play a card.
pub fn can_take_play_card_action(
    game: &GameState,
    side: Side,
    card_id: CardId,
    target: CardTarget,
) -> bool {
    let mut can_play = in_main_phase_with_action_point(game, side)
        && side == card_id.side
        && game.card(card_id).position() == CardPosition::Hand(side)
        && is_valid_target(game, card_id, target)
        && queries::action_cost(game, card_id) <= game.player(side).actions;

    if enters_play_face_up(game, card_id) {
        can_play &= can_pay_card_cost(game, card_id);
    }

    dispatch::perform_query(game, CanPlayCardQuery(card_id), Flag::new(can_play)).into()
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

    let (cost, target_requirement) = match &crate::ability_definition(game, ability_id).ability_type
    {
        AbilityType::Activated(cost, target_requirement) => (cost, target_requirement),
        _ => return false,
    };

    if !matching_targeting(game, target_requirement, ability_id, target) {
        return false;
    }

    let mut can_activate = in_main_phase_with_action_point(game, side)
        && side == ability_id.card_id.side
        && cost.actions <= game.player(side).actions
        && card.is_face_up()
        && card.position().in_play();

    if let Some(custom_cost) = &cost.custom_cost {
        can_activate &= (custom_cost.can_pay)(game, ability_id);
    }

    if let Some(cost) = queries::ability_mana_cost(game, ability_id) {
        can_activate &= cost <= mana::get(game, side, ManaPurpose::ActivateAbility(ability_id));
    }

    dispatch::perform_query(game, CanActivateAbilityQuery(ability_id), Flag::new(can_activate))
        .into()
}

/// Returns true if the `ability_id` ability could be activated with a valid
/// target.
pub fn activated_ability_has_valid_targets(
    game: &GameState,
    side: Side,
    ability_id: AbilityId,
) -> bool {
    match &crate::ability_definition(game, ability_id).ability_type {
        AbilityType::Activated(_, requirement) => match requirement {
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

fn is_valid_target(game: &GameState, card_id: CardId, target: CardTarget) -> bool {
    fn room_can_add(game: &GameState, room_id: RoomId, card_types: Vec<CardType>) -> bool {
        !room_id.is_inner_room()
            && !game
                .occupants(room_id)
                .any(|card| card_types.contains(&crate::get(card.name).card_type))
    }

    let definition = crate::get(game.card(card_id).name);
    if let Some(targeting) = &definition.config.custom_targeting {
        return matching_targeting(game, targeting, card_id, target);
    }

    match definition.card_type {
        CardType::ChampionSpell
        | CardType::Weapon
        | CardType::Artifact
        | CardType::OverlordSpell => target == CardTarget::None,
        CardType::Minion => matches!(target, CardTarget::Room(_)),
        CardType::Project | CardType::Scheme => {
            matches!(target, CardTarget::Room(room_id)
                if room_can_add(game, room_id, vec![CardType::Project, CardType::Scheme]))
        }
        CardType::GameModifier | CardType::Riftcaller => false,
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
        crate::get(game.card(card_id).name).card_type,
        CardType::Minion | CardType::Scheme | CardType::Project
    )
}

/// Returns true if the indicated card should enter play in a room
pub fn enters_play_in_room(game: &GameState, card_id: CardId) -> bool {
    matches!(
        crate::get(game.card(card_id).name).card_type,
        CardType::Minion | CardType::Scheme | CardType::Project
    )
}

/// Returns whether the indicated player can currently take the basic game
/// action to draw a card.
pub fn can_take_draw_card_action(game: &GameState, side: Side) -> bool {
    let can_draw = in_main_phase_with_action_point(game, side) && game.deck(side).next().is_some();
    dispatch::perform_query(game, CanTakeDrawCardActionQuery(side), Flag::new(can_draw)).into()
}

/// Returns whether the indicated player can currently take the basic game
/// action to gain one mana.
pub fn can_take_gain_mana_action(game: &GameState, side: Side) -> bool {
    let can_gain_mana = in_main_phase_with_action_point(game, side);
    dispatch::perform_query(game, CanTakeGainManaActionQuery(side), Flag::new(can_gain_mana)).into()
}

/// Returns whether the indicated player can currently take the basic game
/// action to initiate a raid on the target [RoomId].
pub fn can_take_initiate_raid_action(game: &GameState, side: Side, room_id: RoomId) -> bool {
    let non_empty = room_id.is_inner_room() || game.occupants(room_id).next().is_some();
    let can_initiate = non_empty
        && side == Side::Champion
        && game.info.raid.is_none()
        && in_main_phase_with_action_point(game, side);
    dispatch::perform_query(game, CanInitiateRaidQuery(room_id), Flag::new(can_initiate)).into()
}

/// Returns whether the indicated player can currently take the basic game
/// action to level up a room
pub fn can_take_level_up_room_action(game: &GameState, side: Side, room_id: RoomId) -> bool {
    let has_level_card = game
        .occupants(room_id)
        .chain(game.defenders_unordered(room_id))
        .any(|card| can_level_up_card(game, card.id));
    let can_level_up = has_level_card
        && side == Side::Overlord
        && mana::get(game, side, ManaPurpose::LevelUpRoom(room_id)) > 0
        && in_main_phase_with_action_point(game, side);
    dispatch::perform_query(game, CanLevelUpRoomQuery(room_id), Flag::new(can_level_up)).into()
}

/// Whether the indicated card can be leveled up when the 'level up' action is
/// taken for its room.
pub fn can_level_up_card(game: &GameState, card_id: CardId) -> bool {
    let can_level_up = crate::card_definition(game, card_id).card_type == CardType::Scheme;
    dispatch::perform_query(game, CanLevelUpCardQuery(card_id), Flag::new(can_level_up)).into()
}

/// Whether the indicated card entered play this turn
pub fn entered_play_this_turn(game: &GameState, card_id: CardId) -> bool {
    game.card(card_id).data.last_entered_play == Some(game.info.turn)
}

/// Whether the provided `source` card is able to target the `target` card with
/// an encounter action. Typically used to determine whether a weapon can target
/// a minion, e.g. based on lineage.
pub fn can_encounter_target(game: &GameState, source: CardId, target: CardId) -> bool {
    let can_encounter = matches!(
        (
            crate::card_definition(game, source).config.lineage,
            crate::card_definition(game, target).config.lineage
        ),
        (Some(source_lineage), Some(target_lineage))
        if source_lineage == Lineage::Prismatic ||
            target_lineage == Lineage::Construct ||
            source_lineage == target_lineage
    );

    dispatch::perform_query(
        game,
        CanEncounterTargetQuery(CardEncounter::new(source, target)),
        Flag::new(can_encounter),
    )
    .into()
}

/// Can the `source` card defeat the `target` card in an encounter by paying its
/// shield cost and dealing enough damage to equal its health (potentially after
/// paying mana & applying boosts), or via some other game mechanism?
///
/// This requires [can_encounter_target] to be true.
pub fn can_defeat_target(game: &GameState, source: CardId, target: CardId) -> bool {
    let can_defeat = can_encounter_target(game, source, target)
        && matches!(
            queries::cost_to_defeat_target(game, source, target),
            Some(cost)
            if cost <= mana::get(game, source.side, ManaPurpose::UseWeapon(source))
        );

    dispatch::perform_query(
        game,
        CanDefeatTargetQuery(CardEncounter::new(source, target)),
        Flag::new(can_defeat),
    )
    .into()
}

/// Returns true if the `side` player is in their main phase as described in
/// [in_main_phase] and they have more than zero action points available.
pub fn in_main_phase_with_action_point(game: &GameState, side: Side) -> bool {
    in_main_phase(game, side) && game.player(side).actions > 0
}

/// Returns true if the provided `side` player is currently in their Main phase
/// with no pending prompt responses, and thus can take a primary game action.
pub fn in_main_phase(game: &GameState, side: Side) -> bool {
    can_take_game_actions(game)
        && game.info.turn.side == side
        && game.info.turn_state != TurnState::Ended
        && game.info.raid.is_none()
}

/// Returns true if either player can currently take game standard game actions
/// This generally means the game is currently in progress and neither player is
/// facing a card prompt.
pub fn can_take_game_actions(game: &GameState) -> bool {
    game.info.phase.is_playing()
        && game.overlord.card_prompt_queue.is_empty()
        && game.champion.card_prompt_queue.is_empty()
}

/// Can the Champion choose to not use a weapon ability when encountering
/// the indicated minion card?
pub fn can_take_use_no_weapon_action(game: &GameState, card_id: CardId) -> bool {
    dispatch::perform_query(
        game,
        CanUseNoWeaponQuery(card_id),
        Flag::new(can_take_game_actions(game)),
    )
    .into()
}

/// Can the Champion choose to use the 'End Raid' button to end the access
/// phase of a raid?
pub fn can_take_end_raid_access_phase_action(game: &GameState, raid_id: RaidId) -> bool {
    dispatch::perform_query(
        game,
        CanEndRaidAccessPhaseQuery(raid_id),
        Flag::new(can_take_game_actions(game)),
    )
    .into()
}

/// Is the `side` player currently able to unveil the provided card?
pub fn can_take_unveil_card_action(game: &GameState, side: Side, card_id: CardId) -> bool {
    can_take_game_actions(game)
        && side == Side::Overlord
        && game.card(card_id).is_face_down()
        && game.card(card_id).position().in_play()
        && can_pay_card_cost(game, card_id)
}

/// Returns whether a player can currently take the 'end turn' action.
pub fn can_take_end_turn_action(game: &GameState, side: Side) -> bool {
    in_main_phase(game, side) && game.player(side).actions == 0
}

/// Returns whether a player can currently take the 'start turn' action.
pub fn can_take_start_turn_action(game: &GameState, side: Side) -> bool {
    can_take_game_actions(game)
        && game.info.turn.side == side.opponent()
        && game.info.turn_state == TurnState::Ended
}
