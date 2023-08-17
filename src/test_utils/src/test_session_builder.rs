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

use std::sync::Mutex;

use game_data::player_name::PlayerId;
use game_data::primitives::GameId;
use game_data::tutorial_data::TutorialData;
use maplit::hashmap;
use player_data::{PlayerState, PlayerStatus};

use crate::fake_database::FakeDatabase;
use crate::test_game::TestGame;
use crate::test_helpers;
use crate::test_session::TestSession;

pub struct TestSessionBuilder {
    game: Option<TestGame>,
    do_not_connect: bool,
    game_id: GameId,
    user_id: PlayerId,
    opponent_id: PlayerId,
}

impl TestSessionBuilder {
    pub fn new() -> Self {
        let (game_id, user_id, opponent_id) = test_helpers::generate_ids();
        Self { game: None, do_not_connect: false, game_id, user_id, opponent_id }
    }

    pub fn game(mut self, game: TestGame) -> Self {
        self.game = Some(game);
        self
    }

    pub fn do_not_connect(mut self, do_not_connect: bool) -> Self {
        self.do_not_connect = do_not_connect;
        self
    }

    pub fn game_id(mut self, game_id: GameId) -> Self {
        self.game_id = game_id;
        self
    }

    pub fn user_id(mut self, user_id: PlayerId) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn opponent_id(mut self, opponent_id: PlayerId) -> Self {
        self.opponent_id = opponent_id;
        self
    }

    pub fn build(self) -> TestSession {
        cards_all::initialize();

        if let Some(game) = self.game {
            let database = FakeDatabase {
                generated_game_id: None,
                game: Mutex::new(Some(game.build_game_state_internal(
                    self.game_id,
                    self.user_id,
                    self.opponent_id,
                ))),
                players: Mutex::new(hashmap! {
                    self.user_id => PlayerState {
                        id: self.user_id,
                        status: Some(PlayerStatus::Playing(self.game_id)),
                        adventure: None,
                        tutorial: TutorialData::default()
                    },
                    self.opponent_id => PlayerState {
                        id: self.opponent_id,
                        status: Some(PlayerStatus::Playing(self.game_id)),
                        adventure: None,
                        tutorial: TutorialData::default()
                    }
                }),
            };

            TestSession::new(database, self.user_id, self.opponent_id, !self.do_not_connect)
        } else {
            let database = FakeDatabase {
                generated_game_id: Some(self.game_id),
                game: Mutex::new(None),
                players: Mutex::new(hashmap! {
                    self.user_id => PlayerState {
                        id: self.user_id,
                        status: None,
                        adventure: None,
                        tutorial: TutorialData::default()
                    },
                    self.opponent_id => PlayerState {
                        id: self.opponent_id,
                        status: None,
                        adventure: None,
                        tutorial: TutorialData::default()
                    }
                }),
            };

            TestSession::new(database, self.user_id, self.opponent_id, !self.do_not_connect)
        }
    }
}

impl Default for TestSessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
