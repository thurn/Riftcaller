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
use core_data::game_primitives::{CardId, HasAbilityId, Side};
use game_data::card_state::CardPosition;
use game_data::delegate_data::{DealtDamage, DealtDamageEvent, WillDealDamageEvent};
use game_data::game_state::GameState;
use game_data::random;
use game_data::state_machine_data::{DealDamageData, DealDamageState, DealDamageStep};
use with_error::WithError;

use crate::state_machine::StateMachine;
use crate::{dispatch, mutations, state_machine};

/// Deals damage. Discards random card from the hand of the Champion player for
/// each point of damage. If no cards remain, they lose the game.
pub fn deal(game: &mut GameState, source: impl HasAbilityId, amount: u32) -> Result<()> {
    state_machine::initiate(
        game,
        DealDamageState {
            data: DealDamageData { amount, source: source.ability_id() },
            discarded: vec![],
            step: DealDamageStep::Begin,
        },
    )
}

/// Prevents up to `amount` damage from being dealt to the Champion in the
/// topmost active `deal_damage` state machine.
pub fn prevent(game: &mut GameState, amount: u32) {
    if let Some(damage) = &mut game.state_machines.deal_damage.last_mut() {
        damage.data.amount = damage.data.amount.saturating_sub(amount);
    }
}

/// Returns the amount of damage currently scheduled to be dealt to the
/// Champion in the topmost active `deal_damage` state machine.
pub fn incoming_amount(game: &GameState) -> Option<u32> {
    game.state_machines.deal_damage.last().map(|d| d.data.amount)
}

/// Returns a list of [CardId]s which have been discarded to the topmost active
/// `deal_damage` state machine event, or an empty slice if no such event
/// exists.
pub fn discarded_to_current_event(game: &GameState) -> &[CardId] {
    static NO_CARDS: &Vec<CardId> = &Vec::new();
    game.state_machines
        .deal_damage
        .last()
        .as_ref()
        .map(|state| &state.discarded)
        .unwrap_or(NO_CARDS)
}

/// Run the deal damage state machine, if needed.
pub fn run_state_machine(game: &mut GameState) -> Result<()> {
    state_machine::run::<DealDamageState>(game)
}

impl StateMachine for DealDamageState {
    type Data = DealDamageData;
    type Step = DealDamageStep;

    fn get(game: &GameState) -> &Vec<Self> {
        &game.state_machines.deal_damage
    }

    fn get_mut(game: &mut GameState) -> &mut Vec<Self> {
        &mut game.state_machines.deal_damage
    }

    fn step(&self) -> Self::Step {
        self.step
    }

    fn step_mut(&mut self) -> &mut Self::Step {
        &mut self.step
    }

    fn data(&self) -> Self::Data {
        self.data
    }

    fn evaluate(
        game: &mut GameState,
        step: DealDamageStep,
        data: DealDamageData,
    ) -> Result<Option<DealDamageStep>> {
        Ok(match step {
            DealDamageStep::Begin => Some(DealDamageStep::WillDealDamageEvent),
            DealDamageStep::WillDealDamageEvent => {
                dispatch::invoke_event(
                    game,
                    WillDealDamageEvent(DealtDamage { source: data.source, amount: data.amount }),
                )?;
                Some(DealDamageStep::DiscardCards)
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
                    .last_mut()
                    .with_error(|| "deal_damage")?
                    .discarded = discarded;
                Some(DealDamageStep::DealtDamageEvent)
            }
            DealDamageStep::DealtDamageEvent => {
                dispatch::invoke_event(
                    game,
                    DealtDamageEvent(DealtDamage { source: data.source, amount: data.amount }),
                )?;

                Some(DealDamageStep::Finish)
            }
            DealDamageStep::Finish => {
                game.current_history_counters(Side::Champion).damage_received += data.amount;
                None
            }
        })
    }
}
