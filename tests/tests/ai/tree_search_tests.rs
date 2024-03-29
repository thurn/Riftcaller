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
use ai_testing::nim;
use ai_testing::nim::{NimState, NimWinLossEvaluator};
use ai_tree_search::alpha_beta::AlphaBetaAlgorithm;
use ai_tree_search::minimax::MinimaxAlgorithm;
use tokio::time::Instant;

#[test]
pub fn minimax() {
    let agent = AgentData::omniscient(
        "MINIMAX",
        MinimaxAlgorithm { search_depth: 25 },
        NimWinLossEvaluator {},
    );

    nim::assert_perfect(&NimState::new(1), &agent);
    nim::assert_perfect(&NimState::new(2), &agent);
    nim::assert_perfect(&NimState::new_with_piles(2, 2, 3), &agent);
    nim::assert_perfect(&NimState::new(3), &agent);
    nim::assert_perfect(&NimState::new_with_piles(1, 1, 3), &agent);
    nim::assert_perfect(&NimState::new_with_piles(4, 3, 2), &agent);
}

#[test]
pub fn minimax_deadline_exceeded() {
    let agent = AgentData::omniscient(
        "MINIMAX",
        MinimaxAlgorithm { search_depth: 25 },
        NimWinLossEvaluator {},
    );
    let state = NimState::new(100);
    let start_time = Instant::now();
    let action = agent.pick_action(AgentConfig::with_deadline(1), &state);
    assert!(action.is_ok());
    assert!(start_time.elapsed().as_secs() < 2);
}

#[test]
pub fn alpha_beta() {
    let agent = AgentData::omniscient(
        "ALPHA_BETA",
        AlphaBetaAlgorithm { search_depth: 25 },
        NimWinLossEvaluator {},
    );

    nim::assert_perfect(&NimState::new(1), &agent);
    nim::assert_perfect(&NimState::new(2), &agent);
    nim::assert_perfect(&NimState::new_with_piles(2, 2, 3), &agent);
    nim::assert_perfect(&NimState::new(3), &agent);
    nim::assert_perfect(&NimState::new_with_piles(1, 1, 3), &agent);
    nim::assert_perfect(&NimState::new_with_piles(4, 3, 2), &agent);
    nim::assert_perfect(&NimState::new(5), &agent);
}

#[test]
pub fn alpha_beta_deadline_exceeded() {
    let agent = AgentData::omniscient(
        "ALPHA_BETA",
        AlphaBetaAlgorithm { search_depth: 25 },
        NimWinLossEvaluator {},
    );
    let state = NimState::new(100);
    let start_time = Instant::now();
    let action = agent.pick_action(AgentConfig::with_deadline(1), &state);
    assert!(action.is_ok());
    assert!(start_time.elapsed().as_secs() < 2);
}
