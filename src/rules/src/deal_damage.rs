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
use core_data::game_primitives::{HasAbilityId, Side};
use game_data::card_state::CardPosition;
use game_data::delegate_data::{DealtDamage, DealtDamageEvent, WillDealDamageEvent};
use game_data::game_state::{GamePhase, GameState};
use game_data::history_data::HistoryEvent;
use game_data::random;
use game_data::state_machines::{DealDamageData, DealDamageStep};
use with_error::{verify, WithError};

use crate::{dispatch, mutations};

/// Deals damage. Discards random card from the hand of the Champion player for
/// each point of damage. If no cards remain, they lose the game.
pub fn apply(game: &mut GameState, source: impl HasAbilityId, amount: u32) -> Result<()> {
    verify!(game.state_machines.deal_damage.is_none(), "Damage is already being resolved!");

    game.state_machines.deal_damage = Some(DealDamageData {
        amount,
        source: source.ability_id(),
        discarded: vec![],
        step: DealDamageStep::Begin,
    });

    run_state_machine(game)
}

/// Run the deal damage state machine, if needed.
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    loop {
        if !(game.overlord.prompt_stack.is_empty() & game.champion.prompt_stack.is_empty()) {
            break;
        }

        if game.info.phase != GamePhase::Play {
            break;
        }

        if let Some(data) = &game.state_machines.deal_damage {
            let step = match data.step {
                DealDamageStep::Begin => DealDamageStep::WillDealDamageEvent,
                DealDamageStep::WillDealDamageEvent => {
                    dispatch::invoke_event(
                        game,
                        WillDealDamageEvent(DealtDamage {
                            source: data.source,
                            amount: data.amount,
                        }),
                    )?;
                    DealDamageStep::DiscardCards
                }
                DealDamageStep::DiscardCards => {
                    let mut discarded = vec![];
                    for _ in 0..data.amount {
                        if let Some(card_id) = random::card_in_position(
                            game,
                            Side::Champion,
                            CardPosition::Hand(Side::Champion),
                        ) {
                            mutations::move_card(
                                game,
                                card_id,
                                CardPosition::DiscardPile(Side::Champion),
                            )?;
                            discarded.push(card_id);
                        } else {
                            mutations::game_over(game, Side::Overlord)?;
                        }
                    }

                    game.state_machines
                        .deal_damage
                        .as_mut()
                        .with_error(|| "deal_damage")?
                        .discarded = discarded;

                    DealDamageStep::DealtDamageEvent
                }
                DealDamageStep::DealtDamageEvent => {
                    dispatch::invoke_event(
                        game,
                        DealtDamageEvent(DealtDamage { source: data.source, amount: data.amount }),
                    )?;

                    DealDamageStep::Finish
                }
                DealDamageStep::Finish => {
                    game.add_history_event(HistoryEvent::DealDamage(data.amount));
                    game.state_machines.deal_damage = None;
                    DealDamageStep::Finish
                }
            };

            if let Some(updated) = &mut game.state_machines.deal_damage {
                updated.step = step;
            }
        } else {
            break;
        }
    }
    Ok(())
}
