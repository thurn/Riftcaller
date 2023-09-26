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

use std::time::{Duration, Instant};

use ai_core::agent::AgentConfig;
use ai_core::game_state_node::{GameStateNode, GameStatus};
use ai_game_integration::agents;
use ai_game_integration::state_node::SpelldawnState;
use anyhow::Result;
use clap::{ArgEnum, Parser};
use game_data::game_state::{GameConfiguration, GameState};
use game_data::player_name::{AIPlayer, PlayerId};
use game_data::primitives::{GameId, Side};
use rules::{dispatch, mutations};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Verbosity {
    None,
    Matches,
    Actions,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(arg_enum, value_parser)]
    pub overlord: AIPlayer,
    #[clap(arg_enum, value_parser)]
    pub champion: AIPlayer,
    #[clap(long, value_parser, default_value_t = 5)]
    /// Maximum time in seconds for each agent to use for moves.
    pub move_time: u64,
    #[clap(long, value_parser, default_value_t = 1)]
    /// Number of matches to run between these two named players
    pub matches: u64,
    #[clap(long, value_parser, default_value = "matches")]
    /// How much log output to produce while running
    pub verbosity: Verbosity,
    #[clap(long, value_parser, default_value_t = false)]
    /// Whether to use a deterministic random number generator
    pub deterministic: bool,
    /// Whether to crash the program if a search timeout is exceeded.
    #[clap(long, value_parser, default_value_t = false)]
    pub panic_on_search_timeout: bool,
}

pub fn main() -> Result<()> {
    run(Args::parse())
}

pub fn run(args: Args) -> Result<()> {
    cards_all::initialize();
    let overlord = agents::get(args.overlord);
    let champion = agents::get(args.champion);

    for i in 1..=args.matches {
        if args.verbosity >= Verbosity::Matches {
            println!(">>> Running match {} between {} and {}", i, overlord.name(), champion.name());
        }
        let mut game = GameState::new(
            GameId::new_from_u128(0),
            PlayerId::AI(args.overlord),
            decklists::canonical_deck(Side::Overlord),
            PlayerId::AI(args.champion),
            decklists::canonical_deck(Side::Champion),
            GameConfiguration {
                deterministic: args.deterministic,
                simulation: true,
                ..GameConfiguration::default()
            },
        );
        dispatch::populate_delegate_cache(&mut game);
        mutations::deal_opening_hands(&mut game)?;

        let mut state = SpelldawnState(game);
        if args.verbosity > Verbosity::None {
            println!("Starting game");
        }

        loop {
            match state.status() {
                GameStatus::InProgress { current_turn } => {
                    let agent = if current_turn == Side::Overlord { &overlord } else { &champion };
                    let config = AgentConfig {
                        panic_on_search_timeout: args.panic_on_search_timeout,
                        deadline: Instant::now() + Duration::from_secs(args.move_time),
                    };
                    let action = agent.pick_action(config, &state)?;
                    state.execute_action(current_turn, action)?;
                    clear_action_line(args.verbosity);
                    if args.verbosity > Verbosity::None {
                        println!("{} performs action {:?}", agent.name(), action);
                    }
                }
                GameStatus::Completed { winner } => {
                    let agent = if winner == Side::Overlord { &overlord } else { &champion };
                    if args.verbosity >= Verbosity::Matches {
                        clear_action_line(args.verbosity);
                        println!(
                            "{} wins as {:?}, {} to {}",
                            agent.name(),
                            winner,
                            state.player(winner).score,
                            state.player(winner.opponent()).score
                        );
                    }
                    break;
                }
            }
        }
    }
    Ok(())
}

fn clear_action_line(verbosity: Verbosity) {
    if verbosity == Verbosity::Matches {
        print!("\x1B[1F"); // Moves cursor to beginning of previous line, 1 line up
        print!("\x1B[2K"); // Erase the entire line
    }
}
