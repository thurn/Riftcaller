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

use ai_core::agent::{Agent, AgentConfig, AgentData};
use ai_core::compound_evaluator::CompoundEvaluator;
use ai_monte_carlo::monte_carlo::{MonteCarloAlgorithm, RandomPlayoutEvaluator};
use ai_monte_carlo::uct1::Uct1;
use ai_tree_search::alpha_beta::AlphaBetaAlgorithm;
use ai_tree_search::minimax::MinimaxAlgorithm;
use anyhow::Result;
use game_data::game_actions::GameAction;
use game_data::player_name::AIPlayer;

use with_error::fail;

use crate::evaluators::{
    CardsInHandEvaluator, CardsInPlayEvaluator, ManaDifferenceEvaluator, ProgressCountersEvaluator,
    ScoreEvaluator,
};
use crate::state_node::RiftcallerState;

pub fn get(name: AIPlayer) -> Box<dyn Agent<RiftcallerState>> {
    match name {
        AIPlayer::NoAction => Box::new(NoActionAgent {}),
        AIPlayer::TutorialOpponent => Box::new(AgentData::omniscient(
            "TUTORIAL",
            MinimaxAlgorithm { search_depth: 4 },
            ScoreEvaluator {},
        )),
        AIPlayer::DebugRiftcaller => Box::new(NoActionAgent {}),
        AIPlayer::DebugCovenant => Box::new(NoActionAgent {}),
        AIPlayer::TestMinimax => Box::new(AgentData::omniscient(
            "MINIMAX",
            MinimaxAlgorithm { search_depth: 4 },
            ScoreEvaluator {},
        )),
        AIPlayer::TestAlphaBetaScores => Box::new(AgentData::omniscient(
            "ALPHA_BETA_SCORES",
            AlphaBetaAlgorithm { search_depth: 4 },
            CompoundEvaluator { evaluators: vec![(1, Box::new(ScoreEvaluator {}))] },
        )),
        AIPlayer::BenchmarkAlphaBetaDepth3 => Box::new(AgentData::omniscient(
            "ALPHA_BETA_DEPTH_3",
            AlphaBetaAlgorithm { search_depth: 3 },
            CompoundEvaluator { evaluators: vec![(1, Box::new(ScoreEvaluator {}))] },
        )),
        AIPlayer::TestAlphaBetaHeuristics => Box::new(AgentData::omniscient(
            "ALPHA_BETA_HEURISTICS",
            AlphaBetaAlgorithm { search_depth: 4 },
            CompoundEvaluator {
                evaluators: vec![
                    (100_000, Box::new(ScoreEvaluator {})),
                    (10, Box::new(ManaDifferenceEvaluator {})),
                    (5, Box::new(CardsInHandEvaluator {})),
                    (15, Box::new(CardsInPlayEvaluator {})),
                    (20, Box::new(ProgressCountersEvaluator {})),
                ],
            },
        )),
        AIPlayer::TestUct1 => Box::new(AgentData::omniscient(
            "UCT1",
            MonteCarloAlgorithm { child_score_algorithm: Uct1 {} },
            RandomPlayoutEvaluator {},
        )),
    }
}

pub struct NoActionAgent {}

impl Agent<RiftcallerState> for NoActionAgent {
    fn name(&self) -> &'static str {
        "NO_ACTION"
    }

    fn pick_action(&self, _: AgentConfig, _: &RiftcallerState) -> Result<GameAction> {
        fail!("No Action")
    }

    fn inactive(&self) -> bool {
        true
    }
}
