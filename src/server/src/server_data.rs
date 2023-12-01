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

use core_data::game_primitives::{AdventureId, GameId};
use game_data::player_name::{AIPlayer, PlayerId};
use player_data::PlayerState;
use protos::riftcaller::game_command::Command;
use protos::riftcaller::{ClientMetadata, CommandList, GameCommand};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use with_error::WithError;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SerializedMetadata {
    pub current_adventure: Option<AdventureId>,
    pub current_game: Option<GameId>,
}

#[derive(Debug, Clone)]
pub struct GameResponseOutput {
    /// Response to send to the user who made the initial game request.
    pub user_response: CommandList,
    /// Response to send to update opponent state, if any.
    pub opponent_response: Option<(PlayerId, CommandList)>,
}

/// A response to a user request.
#[derive(Debug, Clone)]
pub struct GameResponse {
    context: ClientData,
    commands: Vec<Command>,
    opponent_response: Option<(PlayerId, Vec<Command>)>,
}

impl GameResponse {
    pub fn new(context: ClientData) -> Self {
        Self { context, commands: vec![], opponent_response: None }
    }

    pub fn command(mut self, command: impl Into<Command>) -> Self {
        self.commands.push(command.into());
        self
    }

    pub fn insert_command(&mut self, index: usize, command: impl Into<Command>) {
        self.commands.insert(index, command.into());
    }

    pub fn push_command(&mut self, command: impl Into<Command>) {
        self.commands.push(command.into());
    }

    pub fn commands(mut self, mut commands: Vec<Command>) -> Self {
        self.commands.append(&mut commands);
        self
    }

    pub fn opponent_response(mut self, opponent_id: PlayerId, commands: Vec<Command>) -> Self {
        if !opponent_id.is_ai_player() {
            self.opponent_response = Some((opponent_id, commands));
        }
        self
    }

    pub fn build(self) -> GameResponseOutput {
        let user_response = CommandList {
            logging_metadata: vec![],
            commands: self.commands.into_iter().map(|c| GameCommand { command: Some(c) }).collect(),
            metadata: Some(self.context.build()),
        };
        let opponent_response = self.opponent_response.map(|(id, commands)| {
            (
                id,
                CommandList {
                    logging_metadata: vec![],
                    commands: commands
                        .into_iter()
                        .map(|c| GameCommand { command: Some(c) })
                        .collect(),
                    metadata: Some(self.context.build()),
                },
            )
        });

        GameResponseOutput { user_response, opponent_response }
    }
}

/// Standard parameters received from a client request
#[derive(Debug, Clone)]
pub struct RequestData {
    pub player_id: PlayerId,
    pub game_id: Option<GameId>,
    pub adventure_id: Option<AdventureId>,
}

/// Standard parameters to send with a client response
#[derive(Debug, Clone, Copy, Default)]
pub struct ClientData {
    pub game_id: Option<GameId>,
    pub adventure_id: Option<AdventureId>,
}

impl ClientData {
    pub fn from_client_metadata(data: Option<&ClientMetadata>) -> anyhow::Result<Self> {
        let metadata = data.with_error(|| "ClientMetadata is required")?;
        let game_id =
            if let Some(g) = &metadata.game_id { Some(GameId::new(parse_ulid(g)?)) } else { None };
        let adventure_id = if let Some(g) = &metadata.adventure_id {
            Some(AdventureId::new(parse_ulid(g)?))
        } else {
            None
        };

        Ok(ClientData { game_id, adventure_id })
    }

    /// Continue using the same client state
    pub fn propagate(request: &RequestData) -> Self {
        Self { game_id: request.game_id, adventure_id: request.adventure_id }
    }

    pub fn with_game_id(data: &RequestData, game_id: Option<GameId>) -> Self {
        Self { game_id, adventure_id: data.adventure_id }
    }

    pub fn with_adventure_id(request: &RequestData, adventure_id: Option<AdventureId>) -> Self {
        Self { game_id: request.game_id, adventure_id }
    }

    pub fn build(self) -> ClientMetadata {
        ClientMetadata {
            adventure_id: self.adventure_id.map(|a| a.to_string()),
            game_id: self.game_id.map(|g| g.to_string()),
        }
    }
}

fn parse_ulid(s: &str) -> anyhow::Result<Ulid> {
    Ulid::from_string(s).with_error(|| format!("Error parsing ID {s}"))
}

/// Describes player data for an oppposing player in a game.
pub enum OpponentData {
    HumanPlayer(Box<PlayerState>),
    NamedPlayer(AIPlayer),
}

impl OpponentData {
    pub fn id(&self) -> PlayerId {
        match self {
            Self::HumanPlayer(p) => p.id,
            OpponentData::NamedPlayer(n) => PlayerId::AI(*n),
        }
    }
}
