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
use game_data::card_state::CardPosition;
use game_data::game_actions::RazeCardActionType;
use game_data::game_state::GameState;
use game_data::primitives::{CardId, CardType, RoomId, Side};
use game_data::raid_data::{RaidChoice, RaidInfo, RaidLabel, RaidStep, ScoredCard};
use game_data::random;
use rules::mana::ManaPurpose;
use rules::{mana, mutations, queries, CardDefinitionExt};

/// Returns a vector of the cards accessed for the current raid target, mutating
/// the [GameState] to store the results of random zone selections.
pub fn select_accessed_cards(game: &mut GameState, info: RaidInfo) -> Result<Vec<CardId>> {
    let target = info.target;

    let accessed = match target {
        RoomId::Vault => mutations::realize_top_of_deck(
            game,
            Side::Overlord,
            queries::vault_access_count(game)?,
        )?,
        RoomId::Sanctum => {
            let count = queries::sanctum_access_count(game)?;

            random::cards_in_position(
                game,
                Side::Overlord,
                CardPosition::Hand(Side::Overlord),
                count as usize,
            )
        }
        RoomId::Crypts => {
            game.card_list_for_position(Side::Overlord, CardPosition::DiscardPile(Side::Overlord))
        }
        _ => game.occupants(target).map(|c| c.id).collect(),
    };

    Ok(accessed)
}

/// Returns a [RaidChoice] for the Champion to access the provided
/// `card_id`, if any action can be taken.
pub fn access_action_for_card(game: &GameState, card_id: CardId) -> Option<RaidChoice> {
    let definition = game.card(card_id).definition();
    match definition.card_type {
        CardType::Scheme if can_score_card(game, card_id) => Some(RaidChoice::new(
            RaidLabel::ScoreCard(card_id),
            RaidStep::StartScoringCard(ScoredCard { id: card_id }),
        )),
        CardType::Project if can_raze_project(game, card_id) => {
            let raze_type = if game.card(card_id).position().in_play() {
                RazeCardActionType::Destroy
            } else {
                RazeCardActionType::Discard
            };
            Some(RaidChoice::new(
                RaidLabel::RazeCard(card_id, raze_type),
                RaidStep::StartRazingCard(card_id, queries::raze_cost(game, card_id)),
            ))
        }
        _ => None,
    }
}

/// Can the Champion player score the `card_id` card when accessed during a
/// raid?
fn can_score_card(game: &GameState, card_id: CardId) -> bool {
    let Some(raid) = &game.raid else {
        return false;
    };

    raid.accessed.contains(&card_id)
        && game.card(card_id).definition().config.stats.scheme_points.is_some()
}

/// Can the Champion player raze the `card_id` project when accessed during a
/// raid?
fn can_raze_project(game: &GameState, card_id: CardId) -> bool {
    !game.card(card_id).position().in_discard_pile()
        && queries::raze_cost(game, card_id)
            <= mana::get(game, Side::Champion, ManaPurpose::RazeCard(card_id))
}