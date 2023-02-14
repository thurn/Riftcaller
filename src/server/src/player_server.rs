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

use anyhow::Result;
use database::Database;
use deck_editor::deck_editor_actions;
use user_action_data::DeckEditorAction;

use crate::requests;
use crate::server_data::{ClientData, GameResponse, RequestData};

pub async fn handle_deck_editor_action(
    database: &mut impl Database,
    data: &RequestData,
    action: &DeckEditorAction,
) -> Result<GameResponse> {
    requests::with_player(database, data, |player| {
        deck_editor_actions::handle(player, action)?;
        Ok(GameResponse::new(ClientData::propagate(data)))
    })
    .await
}
