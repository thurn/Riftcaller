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

//! Tools to facilitate testing. Should be included via wildcard import in all
//! tests.

#![allow(clippy::unwrap_in_result)]

pub mod client_interface;
pub mod fake_database;
pub mod summarize;
pub mod test_adventure;
pub mod test_adventure_helpers;
pub mod test_game;
pub mod test_game_client;
pub mod test_helpers;
pub mod test_interface_helpers;
pub mod test_session;
pub mod test_session_builder;
pub mod test_session_helpers;
pub mod test_world_map;

use adventure_data::adventure::Coins;
use game_data::primitives::{ManaValue, RaidId, RoomId};
use protos::spelldawn::RoomIdentifier;
pub use test_adventure_helpers::TestAdventureHelpers;
pub use test_interface_helpers::{Buttons, TestInterfaceHelpers};
pub use test_session_helpers::TestSessionHelpers;

/// The title returned for hidden cards
pub const HIDDEN_CARD: &str = "Hidden Card";
/// [RoomId] used by default for targeting
pub const ROOM_ID: RoomId = RoomId::RoomA;
/// Client equivalent of [ROOM_ID].
pub const CLIENT_ROOM_ID: RoomIdentifier = RoomIdentifier::RoomA;
/// Default Raid ID to use during testing
pub const RAID_ID: RaidId = RaidId(1);
/// Default mana for players in a test game if not otherwise specified
pub const STARTING_MANA: ManaValue = 999;

pub const STARTING_COINS: Coins = Coins(999);

// fn create_mock_adventure(player_id: PlayerId, side: Side, args:
// AdventureArgs) -> AdventureState {     let battle =
// TileEntity::Battle(BattleData {         opponent_id: AIPlayer::NoAction,
//         opponent_deck: decklists::canonical_deck(side.opponent()),
//         opponent_name: "Opponent Name".to_string(),
//         reward: args.reward,
//         character: CharacterPreset::Overlord,
//         character_facing: CharacterFacing::Down,
//         region_to_reveal: 2,
//     });
//     let mut adventure = mock_adventure::create(
//         AdventureId::new_from_u128(0),
//         AdventureConfiguration {
//             player_id,
//             side,
//             rng: Some(Xoshiro256StarStar::seed_from_u64(314159265358979323)),
//         },
//         decklists::canonical_deck(side),
//         HashMap::new(),
//         None,
//         None,
//         None,
//         Some(battle),
//     );
//     adventure.visiting_position = Some(mock_adventure::BATTLE_POSITION);
//     adventure.coins = args.current_coins;
//     adventure
// }
