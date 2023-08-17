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
pub mod test_game;
pub mod test_helpers;
pub mod test_session;
pub mod test_session_helpers;

use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use game_data::player_name::PlayerId;
use game_data::primitives::{GameId, ManaValue, RaidId, RoomId};
use game_data::tutorial_data::TutorialData;
use maplit::hashmap;
use player_data::PlayerState;
use protos::spelldawn::RoomIdentifier;
pub use test_session_helpers::{Buttons, TestSessionHelpers};

use crate::fake_database::FakeDatabase;
use crate::test_session::TestSession;

pub static NEXT_ID: AtomicU64 = AtomicU64::new(1_000_000);
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

/// Creates an empty [TestSession]. Both provided [PlayerId]s are mapped to
/// empty data. If a game is requested for the session, it will receive the
/// provided [GameId].
pub fn new_session(game_id: GameId, user_id: PlayerId, opponent_id: PlayerId) -> TestSession {
    cards_all::initialize();

    let database = FakeDatabase {
        generated_game_id: Some(game_id),
        game: Mutex::new(None),
        players: Mutex::new(hashmap! {
            user_id => PlayerState {
                id: user_id,
                status: None,
                adventure: None,
                tutorial: TutorialData::default()
            },
            opponent_id => PlayerState {
                id: opponent_id,
                status: None,
                adventure: None,
                tutorial: TutorialData::default()
            }
        }),
    };

    TestSession::new(database, user_id, opponent_id, false)
}

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
