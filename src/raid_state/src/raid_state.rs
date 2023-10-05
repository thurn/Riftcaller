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

pub mod access;
pub mod defenders;
pub mod raid_display_state;
pub mod raid_prompt;

use anyhow::Result;
use game_data::card_state::CardPosition;
use game_data::delegate_data::{
    CardAccessEvent, ChampionScoreCardEvent, EncounterMinionEvent, MinionCombatAbilityEvent,
    MinionDefeatedEvent, RaidAccessSelectedEvent, RaidAccessStartEvent, RaidEvent, RaidOutcome,
    RaidStartEvent, RazeCardEvent, ScoreCard, ScoreCardEvent, UsedWeapon, UsedWeaponEvent,
};
use game_data::game_actions::RaidAction;
use game_data::game_history::HistoryEvent;
use game_data::game_state::{GamePhase, GameState, RaidJumpRequest};
use game_data::game_updates::{GameAnimation, InitiatedBy, TargetedInteraction};
use game_data::primitives::{CardId, GameObjectId, RaidId, RoomId, Side};
use game_data::raid_data::{
    RaidChoice, RaidData, RaidInfo, RaidLabel, RaidState, RaidStatus, RaidStep, ScoredCard,
    WeaponInteraction,
};
use rules::mana::ManaPurpose;
use rules::mutations::SummonMinion;
use rules::{dispatch, flags, mana, mutations, queries, CardDefinitionExt};
use tracing::debug;
use with_error::{fail, verify, WithError};

/// Handle a client request to initiate a new raid. Deducts action points and
/// then invokes [initiate_with_callback].
pub fn handle_initiate_action(
    game: &mut GameState,
    user_side: Side,
    target_room: RoomId,
) -> Result<()> {
    verify!(
        flags::can_take_initiate_raid_action(game, user_side, target_room),
        "Cannot initiate raid for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1)?;
    initiate_with_callback(game, target_room, InitiatedBy::GameAction, |_, _| {})
}

/// Starts a new raid, either as a result of an explicit game action or via a
/// card effect (as differentiated by the [InitiatedBy] prop). Invokes the
/// `on_begin` function immediately with the [RaidId] that will be used for this
/// raid, before any other game logic runs.
pub fn initiate_with_callback(
    game: &mut GameState,
    target: RoomId,
    initiated_by: InitiatedBy,
    on_begin: impl Fn(&mut GameState, RaidId),
) -> Result<()> {
    let raid_id = RaidId(game.info.next_raid_id);
    let raid = RaidData {
        target,
        initiated_by,
        raid_id,
        state: RaidState::Step(RaidStep::Begin),
        encounter: game.defenders_unordered(target).count(),
        accessed: vec![],
        jump_request: None,
    };

    game.info.next_raid_id += 1;
    game.raid = Some(raid);
    on_begin(game, raid_id);
    game.add_animation(|| GameAnimation::InitiateRaid(target, initiated_by));
    game.add_history_event(HistoryEvent::RaidBegin(RaidEvent {
        target,
        raid_id,
        data: initiated_by,
    }));

    run(game, None)
}

/// Run the raid state machine, if needed.
///
/// This will advance the raid state machine through its steps, optionally
/// incorporating a user action choice provided via the `action` parameter.
///
/// The state machine pauses if a player is presented with a prompt to respond
/// to, and aborts if the raid is ended. If no raid is currently active or the
/// state machine cannot currently advance, this function silently ignores the
/// run request.
pub fn run(game: &mut GameState, mut action: Option<RaidAction>) -> Result<()> {
    loop {
        if !(game.overlord.prompt_queue.is_empty() && game.champion.prompt_queue.is_empty()) {
            break;
        }

        if game.info.phase != GamePhase::Play {
            break;
        }

        apply_jump_request_if_needed(game)?;

        if let Some(raid) = &game.raid {
            let info = raid.info();
            match (&raid.state, action) {
                (RaidState::Step(step), _) => {
                    let state = evaluate_raid_step(game, info, *step)?;
                    if let Some(raid) = &mut game.raid {
                        raid.state = state;
                    }
                }
                (RaidState::Prompt(prompt), Some(raid_action)) => {
                    let choice = prompt
                        .choices
                        .get(raid_action.index)
                        .with_error(|| "Index out of bounds")?
                        .clone();
                    let state = evaluate_raid_step(game, info, choice.step)?;
                    if let Some(raid) = &mut game.raid {
                        raid.state = state;
                    }
                    action = None;
                }
                _ => {
                    break;
                }
            }
        } else {
            break;
        }
    }
    Ok(())
}

/// Implements a [RaidJumpRequest], if one has been specified for the current
/// raid.
fn apply_jump_request_if_needed(game: &mut GameState) -> Result<()> {
    if let Some(raid) = &game.raid {
        if let Some(RaidJumpRequest::EncounterMinion(card_id)) = raid.jump_request {
            let (room_id, index) =
                queries::minion_position(game, card_id).with_error(|| "Minion not found")?;
            debug!(?index, ?card_id, ?room_id, "Handling RaidJumpRequest::EncounterMinion");
            let raid = game.raid_mut()?;
            raid.target = room_id;
            raid.encounter = index;
            raid.jump_request = None;
            raid.state = RaidState::Step(RaidStep::EncounterMinion(card_id));
        }
    }

    Ok(())
}

fn evaluate_raid_step(game: &mut GameState, info: RaidInfo, step: RaidStep) -> Result<RaidState> {
    debug!(?step, ?info.target, ?info.raid_id, ?info.encounter, "Evaluating raid step");
    let result = match step {
        RaidStep::Begin => begin_raid(game, info),
        RaidStep::NextEncounter => RaidState::step(defenders::next_encounter(game, info)?),
        RaidStep::PopulateSummonPrompt(minion_id) => populate_summon_prompt(minion_id),
        RaidStep::SummonMinion(minion_id) => summon_minion(game, minion_id),
        RaidStep::DoNotSummon(_) => RaidState::step(RaidStep::NextEncounter),
        RaidStep::EncounterMinion(minion_id) => encounter_minion(game, minion_id),
        RaidStep::PopulateEncounterPrompt(minion_id) => populate_encounter_prompt(game, minion_id),
        RaidStep::UseWeapon(interaction) => use_weapon(game, info, interaction),
        RaidStep::MinionDefeated(interaction) => minion_defeated(game, interaction),
        RaidStep::FireMinionCombatAbility(minion_id) => {
            fire_minion_combat_ability(game, info, minion_id)
        }
        RaidStep::PopulateApproachPrompt => populate_approach_prompt(game),
        RaidStep::AccessStart => access_start(game, info),
        RaidStep::BuildAccessSet => build_access_set(game, info),
        RaidStep::AccessSetBuilt => access_set_built(game, info),
        RaidStep::RevealAccessedCards => reveal_accessed_cards(game, info),
        RaidStep::AccessCards => access_cards(game),
        RaidStep::PopulateAccessPrompt => populate_access_prompt(game, info),
        RaidStep::StartScoringCard(scored) => start_scoring_card(game, scored),
        RaidStep::ChampionScoreEvent(scored) => champion_score_event(game, scored),
        RaidStep::ScoreEvent(scored) => score_event(game, scored),
        RaidStep::ScorePointsForCard(scored) => score_points_for_card(game, scored),
        RaidStep::MoveToScoredPosition(scored) => move_to_scored_position(game, scored),
        RaidStep::StartRazingCard(card_id, cost) => start_razing_card(game, card_id, cost),
        RaidStep::RazeCard(card_id, cost) => raze_card(game, card_id, cost),
        RaidStep::FinishRaid => finish_raid(game),
    };

    // Write history events after each state machine step so they are visible
    // to the next step.
    game.history.write_events();
    result
}

fn begin_raid(game: &mut GameState, info: RaidInfo) -> Result<RaidState> {
    dispatch::invoke_event(
        game,
        RaidStartEvent(RaidEvent { raid_id: info.raid_id, target: info.target, data: () }),
    )?;

    RaidState::step(RaidStep::NextEncounter)
}

fn populate_summon_prompt(minion_id: CardId) -> Result<RaidState> {
    RaidState::prompt(
        RaidStatus::Summon,
        vec![
            RaidChoice::new(RaidLabel::SummonMinion(minion_id), RaidStep::SummonMinion(minion_id)),
            RaidChoice::new(RaidLabel::DoNotSummonMinion, RaidStep::DoNotSummon(minion_id)),
        ],
    )
}

fn summon_minion(game: &mut GameState, minion_id: CardId) -> Result<RaidState> {
    verify!(defenders::can_summon_defender(game, minion_id), "Cannot summon minion");
    mutations::summon_minion(game, minion_id, SummonMinion::PayCosts)?;
    RaidState::step(RaidStep::EncounterMinion(minion_id))
}

fn encounter_minion(game: &mut GameState, minion_id: CardId) -> Result<RaidState> {
    dispatch::invoke_event(game, EncounterMinionEvent(minion_id))?;
    RaidState::step(RaidStep::PopulateEncounterPrompt(minion_id))
}

fn populate_encounter_prompt(game: &mut GameState, minion_id: CardId) -> Result<RaidState> {
    RaidState::prompt(
        RaidStatus::Encounter,
        game.artifacts()
            .filter(|weapon| flags::can_defeat_target(game, weapon.id, minion_id))
            .map(|weapon| {
                let interaction = WeaponInteraction::new(weapon.id, minion_id);
                RaidChoice::new(RaidLabel::UseWeapon(interaction), RaidStep::UseWeapon(interaction))
            })
            .chain(flags::can_take_use_no_weapon_action(game, minion_id).then(|| {
                RaidChoice::new(
                    RaidLabel::DoNotUseWeapon,
                    RaidStep::FireMinionCombatAbility(minion_id),
                )
            }))
            .collect(),
    )
}

fn use_weapon(
    game: &mut GameState,
    info: RaidInfo,
    interaction: WeaponInteraction,
) -> Result<RaidState> {
    let Some(to_defeat) =
        queries::cost_to_defeat_target(game, interaction.weapon_id, interaction.defender_id)
    else {
        fail!("{:?} cannot defeat target: {:?}", interaction.weapon_id, interaction.defender_id)
    };

    mana::spend(
        game,
        Side::Champion,
        ManaPurpose::UseWeapon(interaction.weapon_id),
        to_defeat.cost,
    )?;

    let used_weapon = UsedWeapon {
        weapon_id: interaction.weapon_id,
        target_id: interaction.defender_id,
        mana_spent: to_defeat.cost,
        attack_boost: to_defeat.attack_boost,
    };
    game.add_history_event(HistoryEvent::UseWeapon(info.event(used_weapon)));

    game.add_animation(|| {
        GameAnimation::CombatInteraction(TargetedInteraction {
            source: GameObjectId::CardId(interaction.weapon_id),
            target: GameObjectId::CardId(interaction.defender_id),
        })
    });

    dispatch::invoke_event(game, UsedWeaponEvent(info.event(used_weapon)))?;

    RaidState::step(RaidStep::MinionDefeated(interaction))
}

fn minion_defeated(game: &mut GameState, interaction: WeaponInteraction) -> Result<RaidState> {
    dispatch::invoke_event(game, MinionDefeatedEvent(interaction.defender_id))?;
    RaidState::step(RaidStep::NextEncounter)
}

fn fire_minion_combat_ability(
    game: &mut GameState,
    info: RaidInfo,
    minion_id: CardId,
) -> Result<RaidState> {
    game.add_history_event(HistoryEvent::MinionCombatAbility(info.event(minion_id)));

    game.add_animation(|| {
        GameAnimation::CombatInteraction(TargetedInteraction {
            source: GameObjectId::CardId(minion_id),
            target: GameObjectId::Character(Side::Champion),
        })
    });

    dispatch::invoke_event(game, MinionCombatAbilityEvent(minion_id))?;
    RaidState::step(RaidStep::NextEncounter)
}

fn populate_approach_prompt(game: &mut GameState) -> Result<RaidState> {
    if flags::overlord_has_instant_speed_actions(game) {
        RaidState::prompt(
            RaidStatus::ApproachRoom,
            vec![RaidChoice::new(RaidLabel::ProceedToAccess, RaidStep::AccessStart)],
        )
    } else {
        RaidState::step(RaidStep::AccessStart)
    }
}

fn access_start(game: &mut GameState, info: RaidInfo) -> Result<RaidState> {
    dispatch::invoke_event(game, RaidAccessStartEvent(info.raid_id))?;
    RaidState::step(RaidStep::BuildAccessSet)
}

fn build_access_set(game: &mut GameState, info: RaidInfo) -> Result<RaidState> {
    game.raid_mut()?.accessed = access::select_accessed_cards(game, info)?;
    RaidState::step(RaidStep::AccessSetBuilt)
}

fn access_set_built(game: &mut GameState, info: RaidInfo) -> Result<RaidState> {
    dispatch::invoke_event(
        game,
        RaidAccessSelectedEvent(RaidEvent { raid_id: info.raid_id, target: info.target, data: () }),
    )?;
    RaidState::step(RaidStep::RevealAccessedCards)
}

fn reveal_accessed_cards(game: &mut GameState, info: RaidInfo) -> Result<RaidState> {
    let accessed = game.raid()?.accessed.clone();
    for card_id in &accessed {
        mutations::set_revealed_to(game, *card_id, Side::Champion, true);
    }

    if info.target == RoomId::Sanctum {
        game.add_animation(|| GameAnimation::AccessSanctumCards(accessed))
    }

    RaidState::step(RaidStep::AccessCards)
}

fn access_cards(game: &mut GameState) -> Result<RaidState> {
    let accessed = game.raid()?.accessed.clone();
    for card_id in &accessed {
        dispatch::invoke_event(game, CardAccessEvent(*card_id))?;
    }

    RaidState::step(RaidStep::PopulateAccessPrompt)
}

fn populate_access_prompt(game: &mut GameState, info: RaidInfo) -> Result<RaidState> {
    let can_end = flags::can_take_end_raid_access_phase_action(game, info.raid_id);
    RaidState::prompt(
        RaidStatus::Access,
        game.raid()?
            .accessed
            .iter()
            .filter_map(|card_id| access::access_action_for_card(game, *card_id))
            .chain(can_end.then_some(RaidChoice::new(RaidLabel::EndRaid, RaidStep::FinishRaid)))
            .collect(),
    )
}

fn start_scoring_card(game: &mut GameState, scored: ScoredCard) -> Result<RaidState> {
    mutations::turn_face_up(game, scored.id);
    mutations::move_card(game, scored.id, CardPosition::Scoring)?;
    game.raid_mut()?.accessed.retain(|c| *c != scored.id);
    game.add_animation(|| GameAnimation::ScoreCard(Side::Champion, scored.id));
    RaidState::step(RaidStep::ChampionScoreEvent(scored))
}

fn champion_score_event(game: &mut GameState, scored: ScoredCard) -> Result<RaidState> {
    dispatch::invoke_event(game, ChampionScoreCardEvent(scored.id))?;
    RaidState::step(RaidStep::ScoreEvent(scored))
}

fn score_event(game: &mut GameState, scored: ScoredCard) -> Result<RaidState> {
    dispatch::invoke_event(
        game,
        ScoreCardEvent(ScoreCard { player: Side::Champion, card_id: scored.id }),
    )?;
    RaidState::step(RaidStep::ScorePointsForCard(scored))
}

fn score_points_for_card(game: &mut GameState, scored: ScoredCard) -> Result<RaidState> {
    let scheme_points = game
        .card(scored.id)
        .definition()
        .config
        .stats
        .scheme_points
        .with_error(|| format!("Expected SchemePoints for {:?}", scored.id))?;
    mutations::score_points(game, Side::Champion, scheme_points.points)?;
    RaidState::step(RaidStep::MoveToScoredPosition(scored))
}

fn move_to_scored_position(game: &mut GameState, scored: ScoredCard) -> Result<RaidState> {
    mutations::move_card(game, scored.id, CardPosition::Scored(Side::Champion))?;
    RaidState::step(RaidStep::PopulateAccessPrompt)
}

fn start_razing_card(game: &mut GameState, card_id: CardId, cost: u32) -> Result<RaidState> {
    game.raid_mut()?.accessed.retain(|c| *c != card_id);
    dispatch::invoke_event(game, RazeCardEvent(card_id))?;
    RaidState::step(RaidStep::RazeCard(card_id, cost))
}

fn raze_card(game: &mut GameState, card_id: CardId, cost: u32) -> Result<RaidState> {
    mana::spend(game, Side::Champion, ManaPurpose::RazeCard(card_id), cost)?;
    mutations::move_card(game, card_id, CardPosition::DiscardPile(Side::Overlord))?;
    RaidState::step(RaidStep::PopulateAccessPrompt)
}

fn finish_raid(game: &mut GameState) -> Result<RaidState> {
    mutations::end_raid(game, RaidOutcome::Success)?;
    RaidState::step(RaidStep::FinishRaid)
}
