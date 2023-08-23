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

use anyhow::Result;
use game_data::delegates::{
    EncounterMinionEvent, MinionCombatAbilityEvent, MinionCombatActionsQuery, MinionDefeatedEvent,
    UsedWeapon, UsedWeaponEvent,
};
use game_data::game::{GameState, InternalRaidPhase};
use game_data::game_actions::{EncounterAction, PromptAction};
use game_data::primitives::{CardId, GameObjectId, Side};
use game_data::updates::{GameUpdate, TargetedInteraction};
use rules::mana::ManaPurpose;
use rules::{card_prompt, dispatch, flags, mana, queries};
use with_error::{fail, WithError};

use crate::defenders;
use crate::traits::{RaidDisplayState, RaidPhaseImpl};

/// The primary combat phase of a raid, in which the Champion may use weapon
/// abilities to attempt to defeat an active Overlord minion.
#[derive(Debug, Clone, Copy)]
pub struct EncounterPhase {}

impl RaidPhaseImpl for EncounterPhase {
    type Action = EncounterAction;

    fn unwrap(action: PromptAction) -> Result<EncounterAction> {
        match action {
            PromptAction::EncounterAction(action) => Ok(action),
            _ => fail!("Expected EncounterAction"),
        }
    }

    fn wrap(action: EncounterAction) -> Result<PromptAction> {
        Ok(PromptAction::EncounterAction(action))
    }

    fn enter(self, game: &mut GameState) -> Result<Option<InternalRaidPhase>> {
        dispatch::invoke_event(game, EncounterMinionEvent(game.raid_defender()?))?;
        Ok(None)
    }

    fn actions(self, game: &GameState) -> Result<Vec<EncounterAction>> {
        let defender_id = game.raid_defender()?;
        Ok(game
            .weapons()
            .filter(|weapon| flags::can_defeat_target(game, weapon.id, defender_id))
            .map(|weapon| EncounterAction::UseWeaponAbility(weapon.id, defender_id))
            .chain(minion_combat_actions(game, defender_id))
            .collect())
    }

    fn handle_action(
        self,
        game: &mut GameState,
        action: EncounterAction,
    ) -> Result<Option<InternalRaidPhase>> {
        match action {
            EncounterAction::UseWeaponAbility(source_id, target_id) => {
                let cost = queries::cost_to_defeat_target(game, source_id, target_id)
                    .with_error(|| format!("{source_id:?} cannot defeat target: {target_id:?}"))?;
                mana::spend(game, Side::Champion, ManaPurpose::UseWeapon(source_id), cost)?;

                game.record_update(|| {
                    GameUpdate::TargetedInteraction(TargetedInteraction {
                        source: GameObjectId::CardId(source_id),
                        target: GameObjectId::CardId(target_id),
                    })
                });

                dispatch::invoke_event(
                    game,
                    UsedWeaponEvent(UsedWeapon {
                        raid_id: game.raid()?.raid_id,
                        weapon_id: source_id,
                        target_id,
                        mana_spent: cost,
                    }),
                )?;
                dispatch::invoke_event(game, MinionDefeatedEvent(target_id))?;
            }
            EncounterAction::NoWeapon | EncounterAction::CardAction(_) => {
                let defender_id = game.raid_defender()?;
                game.record_update(|| {
                    GameUpdate::TargetedInteraction(TargetedInteraction {
                        source: GameObjectId::CardId(defender_id),
                        target: GameObjectId::Character(Side::Champion),
                    })
                });
                dispatch::invoke_event(game, MinionCombatAbilityEvent(defender_id))?;
            }
        }

        if let EncounterAction::CardAction(card_action) = action {
            card_prompt::handle(game, Side::Champion, card_action)?;
        }

        defenders::advance_to_next_encounter(game)
    }

    fn display_state(self, game: &GameState) -> Result<RaidDisplayState> {
        defenders::defender_list_display_state(game)
    }
}

/// Actions to present when a minion is encountered in combat in addition to
/// weapon abilities.
fn minion_combat_actions(game: &GameState, minion_id: CardId) -> Vec<EncounterAction> {
    let result = dispatch::perform_query(game, MinionCombatActionsQuery(minion_id), vec![])
        .into_iter()
        .flatten()
        .map(EncounterAction::CardAction)
        .collect::<Vec<_>>();
    if result.is_empty() {
        if flags::can_take_use_no_weapon_action(game, minion_id) {
            vec![EncounterAction::NoWeapon]
        } else {
            vec![]
        }
    } else {
        result
    }
}
