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

use std::fmt;

use clap::ArgEnum;
use convert_case::{Case, Casing};
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use ulid::Ulid;

/// Identifies a player across different games
#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum PlayerId {
    /// ID stored in the database, i.e. a human player
    Database(Ulid),
    /// Known player, i.e. an AI agent.
    AI(AIPlayer),
}

impl PlayerId {
    pub fn generate() -> Self {
        Self::Database(Ulid::new())
    }

    pub fn new(ulid: Ulid) -> Self {
        Self::Database(ulid)
    }

    pub fn is_ai_player(&self) -> bool {
        match self {
            Self::AI(_) => true,
            _ => false,
        }
    }
}

impl fmt::Debug for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerId::Database(id) => write!(f, "{}", id.to_string()),
            PlayerId::AI(name) => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerId::Database(id) => write!(f, "{}", id.to_string()),
            PlayerId::AI(name) => write!(f, "{}", name),
        }
    }
}

/// Identifies a named AI player
#[derive(
    PartialEq, Eq, Hash, Debug, Display, Copy, Clone, Serialize, Deserialize, ArgEnum, Sequence,
)]
pub enum AIPlayer {
    NoAction,
    DebugCovenant,
    DebugRiftcaller,
    TutorialOpponent,
    TestMinimax,
    TestAlphaBetaScores,
    BenchmarkAlphaBetaDepth3,
    TestAlphaBetaHeuristics,
    TestUct1,
}

impl AIPlayer {
    pub fn displayed_name(&self) -> String {
        format!("{self}").from_case(Case::Pascal).to_case(Case::Title)
    }

    pub fn has_no_actions(self) -> bool {
        match self {
            Self::NoAction | Self::DebugCovenant | Self::DebugRiftcaller => true,
            _ => false,
        }
    }
}
