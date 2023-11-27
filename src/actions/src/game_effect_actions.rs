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
use core_data::game_primitives::{GameObjectId, InitiatedBy};
use game_data::animation_tracker::GameAnimation;
use game_data::delegate_data::RaidOutcome;
use game_data::game_effect::GameEffect;
use game_data::game_state::{GameState, RaidJumpRequest};
use game_data::special_effects::SpecialEffect;
use game_data::state_machines::PlayCardOptions;
use raid_state::{custom_access, InitiateRaidOptions};
use rules::{curses, deal_damage, mana, mutations, play_card, CardDefinitionExt};
use with_error::WithError;

use crate::mana::ManaPurpose;

pub fn handle(game: &mut GameState, effect: GameEffect) -> Result<()> {
    match effect {
        GameEffect::Continue => {}
        GameEffect::AbortPlayingCard => mutations::abort_playing_card(game),
        GameEffect::PlayChoiceEffect { owner, target } => {
            let effect = game
                .card(owner)
                .definition()
                .config
                .choice_effect
                .as_ref()
                .with_error(|| "Expected choice_effect")?;

            game.add_animation(|| {
                GameAnimation::CustomEffects(vec![SpecialEffect::TimedEffect {
                    target,
                    effect: effect.clone().owner(GameObjectId::CardId(owner)),
                }])
            })
        }
        GameEffect::SacrificeCard(card_id) => mutations::sacrifice_card(game, card_id)?,
        GameEffect::DestroyCard(card_id) => mutations::destroy_card(game, card_id)?,
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
            mutations::end_raid(game, InitiatedBy::Ability(ability_id), RaidOutcome::Failure)?
        }
        GameEffect::EndCustomAccess(ability_id) => {
            custom_access::end(game, InitiatedBy::Ability(ability_id))?;
        }
        GameEffect::TakeDamageCost(ability_id, amount) => {
            deal_damage::apply(game, ability_id, amount)?
        }
        GameEffect::MoveCard(card_id, target_position) => {
            mutations::move_card(game, card_id, target_position)?
        }
        GameEffect::PreventDamage(amount) => deal_damage::prevent(game, amount),
        GameEffect::PreventCurses(quantity) => curses::prevent_curses(game, quantity),
        GameEffect::SelectCardForPrompt(side, card_id) => {
            game.player_mut(side).prompt_selected_cards.push(card_id);
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
        GameEffect::SetOnPlayState(card_id, state) => {
            game.card_mut(card_id).set_on_play_state(state);
        }
        GameEffect::RecordCardChoice(ability_id, choice) => {
            mutations::record_card_choice(game, ability_id, choice);
        }
        GameEffect::EvadeCurrentEncounter => {
            mutations::apply_raid_jump(game, RaidJumpRequest::EvadeCurrentMinion);
        }
        GameEffect::PlayCardForNoMana(card_id, target) => {
            play_card::initiate(
                game,
                card_id,
                target,
                PlayCardOptions { ignore_action_cost: true, ignore_mana_cost: true },
            )?;
        }
        GameEffect::PreventRaidCardAccess => {
            if let Some(raid) = game.raid.as_mut() {
                raid.is_card_access_prevented = true;
            }
        }
    }

    Ok(())
}
