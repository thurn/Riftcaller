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

use std::collections::HashMap;

use cards::initialize;
use core_ui::actions::InterfaceAction;
use data::player_data::PlayerData;
use data::player_name::PlayerId;
use data::primitives::Side;
use data::tutorial::TutorialData;
use data::user_actions::UserAction;
use maplit::hashmap;
use protos::spelldawn::client_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{ClientAction, CommandList, GameRequest, WorldMapTile};
use server::requests;
use server::requests::GameResponse;

use crate::client_interface::{ClientInterface, HasText};
use crate::fake_database;
use crate::fake_database::FakeDatabase;

pub const EXPLORE_ICON: &str = "icon_app_198";

pub struct TestAdventure {
    pub side: Side,
    pub player_id: PlayerId,
    pub interface: ClientInterface,
    pub map: TestWorldMap,
    pub database: FakeDatabase,
}

impl TestAdventure {
    pub fn new(side: Side) -> Self {
        initialize::run();
        let (game_id, player_id, _) = crate::generate_ids();

        let mut result = Self {
            side,
            player_id,
            interface: ClientInterface::default(),
            map: TestWorldMap::default(),
            database: FakeDatabase {
                generated_game_id: Some(game_id),
                game: None,
                players: hashmap! {
                    player_id => PlayerData {
                        id: player_id,
                        state: None,
                        adventure: None,
                        tutorial: TutorialData::default()
                    }
                },
            },
        };

        result.perform(UserAction::NewAdventure(side));
        result.connect();

        result
    }

    pub fn connect(&mut self) -> CommandList {
        let commands =
            requests::handle_connect(&mut self.database, self.player_id).expect("Connection error");
        self.handle_commands(commands.clone());
        commands
    }

    pub fn perform(&mut self, action: UserAction) -> GameResponse {
        self.perform_client_action(ClientAction { action: Some(action.as_client_action()) })
    }

    pub fn perform_client_action(&mut self, action: ClientAction) -> GameResponse {
        if let Some(Action::StandardAction(standard)) = action.action.as_ref() {
            if let Some(update) = &standard.update {
                // Handle optimistic update
                self.handle_commands(update.clone());
            }

            if standard.payload.is_empty() {
                // Do not send empty payload to server
                return GameResponse::from_commands(vec![]);
            }
        }

        let response = requests::handle_request(
            &mut self.database,
            &GameRequest {
                action: Some(action),
                player_id: Some(fake_database::to_player_identifier(self.player_id)),
                open_panels: vec![],
            },
        )
        .expect("Error handling game request");

        self.handle_commands(response.command_list.clone());

        response
    }

    /// Attempts to find a tile with a sprite containing the substring 'icon'
    /// and then invokes the 'on visit' action for that tile.
    pub fn visit_tile_with_icon(&mut self, icon: impl Into<String>) -> GameResponse {
        let tile = self.map.find_tile_with_sprite(icon);
        let action = tile.tile.on_visit.as_ref().expect("No visit action found");
        self.perform_client_action(action.clone())
    }

    pub fn click_on(&mut self, text: impl Into<String>) -> GameResponse {
        let handlers = self.interface.top_panel().find_handlers(text);
        let action = handlers.expect("Button not found").on_click.expect("OnClick not found");
        self.perform_client_action(action)
    }

    fn handle_commands(&mut self, list: CommandList) {
        for c in list.commands {
            let command = c.command.expect("Command");
            self.interface.update(command.clone());
            self.map.update(command);
        }
    }
}

#[derive(Default)]
pub struct TestWorldMap {
    tiles: HashMap<(i32, i32), TestMapTile>,
}

impl TestWorldMap {
    pub fn update(&mut self, command: Command) {
        if let Command::UpdateWorldMap(map) = command {
            for tile in map.tiles {
                let clone = tile.clone();
                let position = tile.position.expect("tile position").clone();
                self.tiles.insert((position.x, position.y), TestMapTile { tile: clone });
            }
        }
    }

    pub fn tile(&self, x: i32, y: i32) -> &TestMapTile {
        self.tiles.get(&(x, y)).expect("Tile not found")
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    pub fn find_tile_with_sprite(&self, substring: impl Into<String>) -> &TestMapTile {
        let pattern = substring.into();
        self.tiles
            .values()
            .find(move |tile| tile.has_sprite(&pattern))
            .expect("Matching tile not found")
    }
}

pub struct TestMapTile {
    tile: WorldMapTile,
}

impl TestMapTile {
    pub fn has_sprite(&self, substring: &str) -> bool {
        self.tile.sprites.iter().any(|sprite| {
            sprite.sprite_address.as_ref().expect("sprite_address").address.contains(substring)
        })
    }
}
