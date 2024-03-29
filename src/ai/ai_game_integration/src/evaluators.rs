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

use ai_core::state_evaluator::StateEvaluator;
use anyhow::Result;
use core_data::game_primitives::Side;
use game_data::card_state::CardCounter;

use rules::{mana, queries};
use rules::mana::ManaPurpose;

use crate::state_node::RiftcallerState;

pub struct ScoreEvaluator {}

impl StateEvaluator<RiftcallerState> for ScoreEvaluator {
    fn evaluate(&self, node: &RiftcallerState, side: Side) -> Result<i32> {
        Ok(queries::score(node, side) as i32 - (queries::score(node, side.opponent()) as i32))
    }
}

pub struct ManaDifferenceEvaluator {}

impl StateEvaluator<RiftcallerState> for ManaDifferenceEvaluator {
    fn evaluate(&self, game: &RiftcallerState, side: Side) -> Result<i32> {
        Ok(mana::get(game, side, ManaPurpose::AllSources) as i32
            - mana::get(game, side.opponent(), ManaPurpose::AllSources) as i32)
    }
}

pub struct CardsInHandEvaluator {}

impl StateEvaluator<RiftcallerState> for CardsInHandEvaluator {
    fn evaluate(&self, game: &RiftcallerState, side: Side) -> Result<i32> {
        Ok(game.hand(side).count() as i32)
    }
}

pub struct CardsInPlayEvaluator {}

impl StateEvaluator<RiftcallerState> for CardsInPlayEvaluator {
    fn evaluate(&self, game: &RiftcallerState, side: Side) -> Result<i32> {
        Ok(game.cards(side).iter().filter(|c| c.position().in_play()).count() as i32)
    }
}

pub struct ProgressCountersEvaluator {}

impl StateEvaluator<RiftcallerState> for ProgressCountersEvaluator {
    fn evaluate(&self, game: &RiftcallerState, side: Side) -> Result<i32> {
        if side == Side::Riftcaller {
            return Ok(0);
        }

        Ok(game
            .cards(side)
            .iter()
            .filter(|c| c.position().in_play())
            .map(|c| c.counters(CardCounter::Progress))
            .sum::<u32>() as i32)
    }
}
