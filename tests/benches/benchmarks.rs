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

use std::time::Duration;

use actions::legal_actions;
use ai_core::agent::{Agent, AgentConfig, AgentData};
use ai_game_integration::evaluators::ScoreEvaluator;
use ai_game_integration::state_node::SpelldawnState;
use ai_monte_carlo::monte_carlo::{MonteCarloAlgorithm, RandomPlayoutEvaluator};
use ai_monte_carlo::uct1::Uct1;
use ai_testing::nim::{NimState, NimWinLossEvaluator};
use ai_testing::run_matchup_impl::{self, Args, Verbosity};
use ai_tree_search::alpha_beta::AlphaBetaAlgorithm;
use ai_tree_search::minimax::MinimaxAlgorithm;
use anyhow::Result;
use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use game_data::game::{GameConfiguration, GameState, MulliganDecision};
use game_data::game_actions::{GameAction, GameStateAction};
use game_data::player_name::{AIPlayer, PlayerId};
use game_data::primitives::{GameId, Side};
use rules::{dispatch, mutations};

criterion_group!(
    benches,
    run_matchup,
    legal_actions,
    minimax_nim,
    alpha_beta_nim,
    uct1_nim,
    uct1_search,
    alpha_beta_search
);
criterion_main!(benches);

fn configure(group: &mut BenchmarkGroup<WallTime>) {
    cards_all::initialize();
    group.confidence_level(0.99).noise_threshold(0.025).measurement_time(Duration::from_secs(60));
}

pub fn run_matchup(c: &mut Criterion) {
    let mut group = c.benchmark_group("run_matchup");
    configure(&mut group);
    group.bench_function("run_matchup", |b| {
        b.iter(|| {
            let _result = run_matchup_impl::run(Args {
                overlord: AIPlayer::BenchmarkAlphaBetaDepth3,
                champion: AIPlayer::BenchmarkAlphaBetaDepth3,
                move_time: 3600,
                matches: 1,
                verbosity: Verbosity::None,
                deterministic: true,
                panic_on_search_timeout: true,
            });
        })
    });
    group.finish();
}

pub fn legal_actions(c: &mut Criterion) {
    let mut group = c.benchmark_group("legal_actions");
    configure(&mut group);
    let game = create_canonical_game().unwrap();
    group.bench_function("legal_actions", |b| {
        b.iter(|| {
            let _actions =
                legal_actions::evaluate(&game, Side::Overlord).unwrap().collect::<Vec<_>>();
        })
    });
    group.finish();
}

pub fn minimax_nim(c: &mut Criterion) {
    let mut group = c.benchmark_group("minimax_nim");
    configure(&mut group);
    let state = NimState::new(4);
    let agent = AgentData::omniscient(
        "MINIMAX",
        MinimaxAlgorithm { search_depth: 25 },
        NimWinLossEvaluator {},
    );

    group.bench_function("minimax_nim", |b| {
        b.iter(|| {
            agent.pick_action(AgentConfig::with_deadline(10), &state).expect("Error running agent");
        })
    });
    group.finish();
}

pub fn alpha_beta_nim(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_beta_nim");
    configure(&mut group);
    let state = NimState::new(5);
    let agent = AgentData::omniscient(
        "ALPHA_BETA",
        AlphaBetaAlgorithm { search_depth: 25 },
        NimWinLossEvaluator {},
    );

    group.bench_function("alpha_beta_nim", |b| {
        b.iter(|| {
            agent.pick_action(AgentConfig::with_deadline(10), &state).expect("Error running agent");
        })
    });
    group.finish();
}

pub fn uct1_nim(c: &mut Criterion) {
    let mut group = c.benchmark_group("uct1_nim");
    configure(&mut group);
    let state = NimState::new(5);
    let evaluator = RandomPlayoutEvaluator {};
    let player = state.turn;
    let monte_carlo = MonteCarloAlgorithm { child_score_algorithm: Uct1 {} };

    group.bench_function("uct1_nim", |b| {
        b.iter(|| {
            monte_carlo
                .run_search(|i| i == 10_000, &state, &evaluator, player)
                .expect("run_search() Error");
        })
    });
    group.finish();
}

pub fn uct1_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("uct1_search");
    configure(&mut group);
    let game = SpelldawnState(create_canonical_game().unwrap());
    let evaluator = RandomPlayoutEvaluator {};
    let monte_carlo = MonteCarloAlgorithm { child_score_algorithm: Uct1 {} };

    group.bench_function("uct1_search", |b| {
        b.iter(|| {
            monte_carlo
                .run_search(|i| i == 1000, &game, &evaluator, Side::Overlord)
                .expect("run_search() Error");
        })
    });
}

pub fn alpha_beta_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_beta_search");
    configure(&mut group);
    let game = SpelldawnState(create_canonical_game().unwrap());
    let agent = AgentData::omniscient(
        "ALPHA_BETA",
        AlphaBetaAlgorithm { search_depth: 3 },
        ScoreEvaluator {},
    );

    group.bench_function("alpha_beta_search", |b| {
        b.iter(|| {
            agent.pick_action(AgentConfig::with_deadline(10), &game).expect("Error running agent");
        })
    });
    group.finish();
}

/// Creates a new deterministic game using the canonical decklists, deals
/// opening hands and resolves mulligans.
fn create_canonical_game() -> Result<GameState> {
    let mut game = GameState::new(
        GameId::new_from_u128(0),
        PlayerId::AI(AIPlayer::NoAction),
        decklists::CANONICAL_OVERLORD.clone(),
        PlayerId::AI(AIPlayer::NoAction),
        decklists::CANONICAL_CHAMPION.clone(),
        GameConfiguration { deterministic: true, simulation: true, scripted_tutorial: false },
    );

    dispatch::populate_delegate_cache(&mut game);
    mutations::deal_opening_hands(&mut game)?;
    actions::handle_game_action(
        &mut game,
        Side::Overlord,
        &GameAction::GameStateAction(GameStateAction::MulliganDecision(MulliganDecision::Keep)),
    )?;
    actions::handle_game_action(
        &mut game,
        Side::Champion,
        &GameAction::GameStateAction(GameStateAction::MulliganDecision(MulliganDecision::Keep)),
    )?;

    Ok(game)
}
