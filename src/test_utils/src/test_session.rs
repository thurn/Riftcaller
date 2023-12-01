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

//! A fake game client. Records server responses about a game and stores them in
//! [TestSession].

use actions::legal_actions;
use adventure_data::adventure::TileState;
use anyhow::Result;
use core_data::adventure_primitives::{Coins, TilePosition};
use core_data::game_primitives::{GameId, Side};
use game_data::card_name::{CardName, CardVariant};
use game_data::card_state::CardPosition;
use game_data::game_actions::GameAction;
#[allow(unused_imports)] // Used in docs
use game_data::game_state::GameState;
use game_data::player_name::PlayerId;
use protos::spelldawn::client_action::Action;
use protos::spelldawn::{CardIdentifier, ClientAction, ClientMetadata, CommandList, GameRequest};
use rules::mutations;
use server::ai_agent_response;
use server::server_data::{ClientData, GameResponse, GameResponseOutput};
use with_error::WithError;
use {adapters, tokio};

use crate::fake_database::FakeDatabase;
use crate::test_game_client::{self, TestGameClient};
use crate::{fake_database, TestSessionHelpers};

/// A helper for interacting with a database and server calls during testing.
///
/// This struct keeps track of server responses and converts them into a useful
/// format for writing tests. This enables our 'black box' testing strategy,
/// where the game is almost exclusively tested via the public client-facing
/// API.
///
/// There are actually two perspectives on an ongoing game: each player has
/// their own view of the state of the game, which differs due to hidden
/// information. This struct has two different [TestGameClient]s which get
/// updated based on server responses, representing what the two players are
/// seeing.
pub struct TestSession {
    /// This is the perspective of the player identified by the `user_id`
    /// parameter to [Self::new].
    pub client: TestGameClient,
    /// This is the perspective of the player identified by the `opponent_id`
    /// parameter to [Self::new].
    pub opponent: TestGameClient,

    metadata: ClientMetadata,
    database: FakeDatabase,
}

impl TestSession {
    /// Creates a new session with the provided test database.
    pub fn new(
        database: FakeDatabase,
        user_id: PlayerId,
        opponent_id: PlayerId,
        connect: bool,
    ) -> Self {
        let mut result = Self {
            client: TestGameClient::new(user_id),
            opponent: TestGameClient::new(opponent_id),
            metadata: ClientMetadata::default(),
            database,
        };

        if connect {
            result.connect(user_id).expect("Connection failed");
            result.connect(opponent_id).expect("Connection failed");
        }

        result
    }

    pub fn game_id(&self) -> GameId {
        self.database.game().id
    }

    /// Simulates a client connecting to the server.
    ///
    /// Returns the commands which would be sent to the client when connected.
    #[tokio::main]
    pub async fn connect(&mut self, user_id: PlayerId) -> Result<CommandList> {
        let result = server::handle_connect(&self.database, user_id).await?.build();
        let to_update = match () {
            _ if user_id == self.client.id => &mut self.client,
            _ if user_id == self.opponent.id => &mut self.opponent,
            _ => panic!("Unknown user id: {user_id:?}"),
        };

        // Clear all previous state
        *to_update = TestGameClient::new(user_id);

        if let Some(m) = result.user_response.metadata.clone() {
            self.metadata = m;
        }

        for command in result.user_response.commands.iter() {
            let c = command.command.as_ref().with_error(|| "command")?;
            to_update.handle_command(c);
        }

        Ok(result.user_response)
    }

    /// Execute a simulated client request for this game as a specific user,
    /// updating the client state as appropriate based on the responses.
    /// Returns the [GameResponseOutput] for this action or an error if the
    /// server request failed.
    #[tokio::main]
    pub async fn perform_action(
        &mut self,
        action: Action,
        player_id: PlayerId,
    ) -> Result<GameResponseOutput> {
        let metadata = self.metadata.clone();

        if let Action::StandardAction(standard) = &action {
            if let Some(update) = &standard.update {
                // Handle optimistic update
                self.client.handle_command_list(update.clone());
            }

            if standard.payload.is_empty() {
                // Do not send empty payload to server
                return Ok(GameResponse::new(ClientData::default()).build());
            }
        }

        let response = server::handle_action(
            &self.database,
            player_id,
            &GameRequest {
                action: Some(ClientAction { action: Some(action) }),
                player_id: Some(fake_database::to_player_identifier(player_id)),
                open_panels: vec![],
                metadata: Some(metadata),
            },
        )
        .await?
        .build();

        if let Some(m) = response.user_response.metadata.clone() {
            self.metadata = m;
        }

        let (opponent_id, local, remote) = self.opponent_local_remote(player_id);

        for command in &response.user_response.commands {
            let c = command.command.as_ref().with_error(|| "command")?;
            local.handle_command(c);
        }

        if let Some((channel_user_id, list)) = &response.opponent_response {
            assert_eq!(*channel_user_id, opponent_id);
            for command in &list.commands {
                let c = command.command.as_ref().with_error(|| "command")?;
                remote.handle_command(c);
            }
        }

        Ok(response)
    }

    /// Adds a named card to its owner's hand.
    ///
    /// This function operates by locating a test card in the owner's deck and
    /// overwriting it with the provided `card_name`. This card is then
    /// moved to the user's hand via [GameState::move_card_internal]. The
    /// complete game state is synced for both players by invoking
    /// [Self::connect].
    ///
    /// This function will *not* spend action points, check the legality of
    /// drawing a card, invoke any game events, or append a game update. It
    /// will correctly update the card's sorting key, however.
    ///
    /// Returns the client [CardIdentifier] for the drawn card. Panics if no
    /// test cards remain in the user's deck.
    pub fn add_to_hand(&mut self, card_name: CardName) -> CardIdentifier {
        self.add_variant_to_hand(CardVariant::standard(card_name))
    }

    /// Equivalent method to [Self::add_to_hand] which takes a [CardVariant].
    pub fn add_variant_to_hand(&mut self, card_variant: CardVariant) -> CardIdentifier {
        let side = test_game_client::side_for_card_name(card_variant.name);
        let card_id = self
            .database
            .game()
            .cards_in_position(side, CardPosition::DeckUnknown(side))
            .filter(|c| c.variant.name.is_test_card())
            .last() // Use last to avoid overwriting 'next draw' configuration
            .unwrap()
            .id;
        self.database.mutate_game(|game| {
            test_game_client::overwrite_card(game, card_id, card_variant);
            game.move_card_internal(card_id, CardPosition::Hand(side));
            mutations::set_visible_to(game, card_id, card_id.side, true);
        });

        self.connect(self.client.id).expect("User connection error");
        self.connect(self.opponent.id).expect("Opponent connection error");

        adapters::card_identifier(card_id)
    }

    /// Inserts an adventure tile in the indicated tile `position`. Panics if
    /// there is no currently-active adventure in the database.
    pub fn overwrite_adventure_tile(&mut self, position: TilePosition, state: TileState) {
        self.database.mutate_player(self.client.id, |player| {
            player
                .adventure
                .as_mut()
                .expect("No active adventure")
                .tiles
                .insert(position, state.clone());
        });

        self.connect(self.client.id).expect("User connection error");
    }

    /// Looks up the [PlayerId] for the [Side] player.
    pub fn player_id_for_side(&self, side: Side) -> PlayerId {
        if self.database.game().player(side).id == self.client.id {
            self.client.id
        } else if self.database.game().player(side).id == self.opponent.id {
            self.opponent.id
        } else {
            panic!("Cannot find PlayerId for side {side:?}")
        }
    }

    /// Equivalent to [legal_actions] but returns a [Result] instead of panic on
    /// error
    pub fn legal_actions_result(&self, side: Side) -> Result<Vec<GameAction>> {
        let game = self.database.game.lock().unwrap();
        let actions = legal_actions::evaluate(game.as_ref().expect("game"), side)?;
        Ok(actions.collect())
    }

    /// Evaluates legal actions for the [Side] player in the current game state.
    pub fn legal_actions(&self, side: Side) -> Vec<GameAction> {
        legal_actions::evaluate(self.database.game.lock().unwrap().as_ref().expect("game"), side)
            .expect("Error evaluating legal actions")
            .collect()
    }

    #[tokio::main]
    pub async fn run_agent_loop(&mut self) {
        let (game_id, user_id) = (self.game_id(), self.user_id());
        ai_agent_response::run_agent_loop_for_tests(&self.database, game_id, user_id)
            .await
            .expect("Error running agent loop");
    }

    /// Returns the number of Coins the current player has in their active
    /// adventure
    pub fn current_coins(&self) -> Coins {
        let db = self.database.players.lock().unwrap();
        let player = db.get(&self.user_id());
        player.unwrap().adventure.as_ref().unwrap().coins
    }

    /// Returns a triple of (opponent_id, local_client, remote_client) for the
    /// provided player ID
    fn opponent_local_remote(
        &mut self,
        player_id: PlayerId,
    ) -> (PlayerId, &mut TestGameClient, &mut TestGameClient) {
        match () {
            _ if player_id == self.client.id => {
                (self.opponent.id, &mut self.client, &mut self.opponent)
            }
            _ if player_id == self.opponent.id => {
                (self.client.id, &mut self.opponent, &mut self.client)
            }
            _ => panic!("Unknown user id: {player_id:?}"),
        }
    }
}
