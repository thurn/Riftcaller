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

use anyhow::Result;
use card_definition_data::cards::CardDefinitionExt;
use core_data::game_primitives::{GameObjectId, InitiatedBy};
use game_data::animation_tracker::GameAnimation;
use game_data::card_state::CardCounter;
use game_data::delegate_data::RaidOutcome;
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::prompt_data::PromptData;
use game_data::raid_data::RaidJumpRequest;
use game_data::special_effects::SpecialEffect;
use game_data::state_machine_data::PlayCardOptions;
use rules::raids::raid_state::InitiateRaidOptions;
use rules::raids::{custom_access, raid_state};
use rules::{curses, damage, destroy, draw_cards, end_raid, mana, mutations, play_card, prompts};
use with_error::WithError;

use crate::mana::ManaPurpose;

pub fn handle(game: &mut GameState, effect: GameEffect) -> Result<()> {
    match effect {
        GameEffect::Continue => {}
        GameEffect::AbortPlayingCard => play_card::abort(game),
        GameEffect::PlayChoiceEffect { owner, target } => {
            let effect = game
                .card(owner)
                .definition()
                .config
                .visual_effect
                .as_ref()
                .with_error(|| "Expected choice_effect")?;

            game.add_animation(|| {
                GameAnimation::CustomEffects(vec![SpecialEffect::TimedEffect {
                    target,
                    effect: effect.clone().owner(GameObjectId::CardId(owner)),
                }])
            })
        }
        GameEffect::DrawCards(side, count, initiated_by) => {
            draw_cards::run(game, side, count, initiated_by)?;
        }
        GameEffect::SacrificeCard(card_id) => mutations::sacrifice_card(game, card_id)?,
        GameEffect::DestroyCard(card_id, initiated_by) => {
            destroy::run(game, vec![card_id], initiated_by)?
        }
        GameEffect::ManaCost(side, amount, initiated_by) => {
            mana::spend(game, side, initiated_by, ManaPurpose::PayForTriggeredAbility, amount)?
        }
        GameEffect::ActionCost(side, amount) => mutations::spend_action_points(game, side, amount)?,
        GameEffect::InitiateRaid(room_id, ability_id) => raid_state::initiate_with_callback(
            game,
            room_id,
            InitiatedBy::Ability(ability_id),
            InitiateRaidOptions::default(),
            |_, _| {},
        )?,
        GameEffect::EndRaid(ability_id) => {
            end_raid::run(game, InitiatedBy::Ability(ability_id), RaidOutcome::Failure)?
        }
        GameEffect::EndCustomAccess(ability_id) => {
            custom_access::end(game, InitiatedBy::Ability(ability_id))?;
        }
        GameEffect::TakeDamageCost(ability_id, amount) => damage::deal(game, ability_id, amount)?,
        GameEffect::MoveCard(card_id, target_position) => {
            mutations::move_card(game, card_id, target_position)?
        }
        GameEffect::PreventDamage(amount) => damage::prevent(game, amount),
        GameEffect::PreventCurses(quantity) => curses::prevent_curses(game, quantity),
        GameEffect::PreventDestroyingCard(id) => destroy::prevent(game, id),
        GameEffect::SelectCardForPrompt(side, card_id) => {
            game.player_mut(side).prompt_selected_cards.push(card_id);
        }
        GameEffect::ClearAllSelectedCards(side) => {
            game.player_mut(side).prompt_selected_cards.clear();
        }
        GameEffect::PushPromptWithIndex(side, ability_id, index) => {
            prompts::push_with_data(game, side, ability_id, PromptData::Index(index));
        }
        GameEffect::SwapWithSelected(side, card_id) => {
            let source_position = game.card(card_id).position();
            let target = game
                .player_mut(side)
                .prompt_selected_cards
                .pop()
                .with_error(|| "No card selected")?;
            let target_position = game.card(target).position();
            mutations::move_card(game, card_id, target_position)?;
            mutations::move_card(game, target, source_position)?;
        }
        GameEffect::AppendCustomCardState(card_id, state) => {
            game.card_mut(card_id).custom_state.push(state);
        }
        GameEffect::EvadeCurrentEncounter => {
            mutations::evade_current_minion(game)?;
        }
        GameEffect::PlayCardForNoMana(card_id, target, from_zone, initiated_by) => {
            play_card::initiate(
                game,
                card_id.side,
                card_id,
                target,
                from_zone,
                initiated_by,
                PlayCardOptions {
                    ignore_action_cost: true,
                    ignore_mana_cost: true,
                    ignore_phase: true,
                    ignore_position: true,
                },
            )?;
        }
        GameEffect::ChangeRaidTarget(room_id, _) => {
            mutations::apply_raid_jump(game, RaidJumpRequest::ChangeTarget(room_id));
        }
        GameEffect::PreventRaidCardAccess => {
            if let Some(raid) = game.raid.as_mut() {
                raid.is_card_access_prevented = true;
            }
        }
        GameEffect::RevealCard(card_id) => {
            mutations::reveal_card(game, card_id)?;
        }
        GameEffect::AddPowerCharges(card_id, count) => {
            game.card_mut(card_id).add_counters(CardCounter::PowerCharges, count);
        }
    }

    Ok(())
}
