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

use ai_core::state_evaluator::StateEvaluator;
use anyhow::Result;
use game_data::card_state::CardCounter;
use game_data::primitives::Side;
use rules::mana::ManaPurpose;
use rules::{mana, queries};

use crate::state_node::SpelldawnState;

pub struct ScoreEvaluator {}

impl StateEvaluator<SpelldawnState> for ScoreEvaluator {
    fn evaluate(&self, node: &SpelldawnState, side: Side) -> Result<i32> {
        Ok(queries::score(node, side) as i32 - (queries::score(node, side.opponent()) as i32))
    }
}

pub struct ManaDifferenceEvaluator {}

impl StateEvaluator<SpelldawnState> for ManaDifferenceEvaluator {
    fn evaluate(&self, game: &SpelldawnState, side: Side) -> Result<i32> {
        Ok(mana::get(game, side, ManaPurpose::AllSources) as i32
            - mana::get(game, side.opponent(), ManaPurpose::AllSources) as i32)
    }
}

pub struct CardsInHandEvaluator {}

impl StateEvaluator<SpelldawnState> for CardsInHandEvaluator {
    fn evaluate(&self, game: &SpelldawnState, side: Side) -> Result<i32> {
        Ok(game.hand(side).count() as i32)
    }
}

pub struct CardsInPlayEvaluator {}

impl StateEvaluator<SpelldawnState> for CardsInPlayEvaluator {
    fn evaluate(&self, game: &SpelldawnState, side: Side) -> Result<i32> {
        Ok(game.cards(side).iter().filter(|c| c.position().in_play()).count() as i32)
    }
}

pub struct ProgressCountersEvaluator {}

impl StateEvaluator<SpelldawnState> for ProgressCountersEvaluator {
    fn evaluate(&self, game: &SpelldawnState, side: Side) -> Result<i32> {
        if side == Side::Champion {
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
