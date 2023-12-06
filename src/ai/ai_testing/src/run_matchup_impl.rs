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

use std::time::{Duration, Instant};

use ai_core::agent::AgentConfig;
use ai_core::game_state_node::{GameStateNode, GameStatus};
use ai_game_integration::agents;
use ai_game_integration::state_node::RiftcallerState;
use anyhow::Result;
use clap::{ArgEnum, Parser};
use core_data::game_primitives::{GameId, Side};
use game_data::game_state::{GameConfiguration, GameState};
use game_data::player_name::{AIPlayer, PlayerId};

use rules::{dispatch, mutations, queries};

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
    pub covenant: AIPlayer,
    #[clap(arg_enum, value_parser)]
    pub riftcaller: AIPlayer,
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
    let covenant = agents::get(args.covenant);
    let riftcaller = agents::get(args.riftcaller);

    for i in 1..=args.matches {
        if args.verbosity >= Verbosity::Matches {
            println!(">>> Running match {} between {} and {}", i, covenant.name(), riftcaller.name());
        }
        let mut game = GameState::new(
            GameId::new_from_u128(0),
            PlayerId::AI(args.covenant),
            decklists::canonical_deck(Side::Covenant),
            PlayerId::AI(args.riftcaller),
            decklists::canonical_deck(Side::Riftcaller),
            GameConfiguration {
                deterministic: args.deterministic,
                simulation: true,
                ..GameConfiguration::default()
            },
        );
        dispatch::populate_delegate_cache(&mut game);
        mutations::deal_opening_hands(&mut game)?;

        let mut state = RiftcallerState(game);
        if args.verbosity > Verbosity::None {
            println!("Starting game");
        }

        loop {
            match state.status() {
                GameStatus::InProgress { current_turn } => {
                    let agent = if current_turn == Side::Covenant { &covenant } else { &riftcaller };
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
                    let agent = if winner == Side::Covenant { &covenant } else { &riftcaller };
                    if args.verbosity >= Verbosity::Matches {
                        clear_action_line(args.verbosity);
                        println!(
                            "{} wins as {:?}, {} to {}",
                            agent.name(),
                            winner,
                            queries::score(&state, winner),
                            queries::score(&state, winner.opponent())
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
