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

use adventure_data::adventure::BattleData;
use core_data::adventure_primitives::Coins;
use core_data::game_primitives::Side;
use game_data::player_name::AIPlayer;

pub fn create(side: Side) -> BattleData {
    BattleData {
        opponent_id: AIPlayer::TestAlphaBetaHeuristics,
        opponent_deck: decklists::canonical_deck(side),
        opponent_name: "Cloaked Bandit".to_string(),
        reward: Coins(250),
    }
}
