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

use adapters::response_builder::ResponseBuilder;
use adapters::CustomCardIdentifier;
use card_definition_data::card_view_context::CardViewContext;
use card_definition_data::cards::CardDefinitionExt;
use core_data::game_primitives::{CardId, School};
use game_data::card_state::CardState;
use game_data::delegate_data::{CardStatusMarker, CardStatusMarkersQuery};
use game_data::game_state::GameState;
use protos::riftcaller::{
    CardIdentifier, CardPrefab, CardTitle, CardView, RevealedCardView, RulesText,
};
use rules::dispatch;

use crate::positions;

/// Builds Status Marker cards, token cards which are displayed stacked
/// underneath cards in the arena to indicate ongoing status effects.
pub fn build(builder: &ResponseBuilder, game: &GameState, card: &CardState) -> Vec<CardView> {
    if !card.position().in_play() {
        vec![]
    } else {
        dispatch::perform_query(game, CardStatusMarkersQuery(&card.id), vec![])
            .into_iter()
            .map(|marker| marker_card(builder, game, card, marker, card.id))
            .collect()
    }
}

fn marker_card(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
    marker: CardStatusMarker,
    target: CardId,
) -> CardView {
    let source_side = marker.source.side();
    let source_definition = game.card(marker.source.card_id).definition();

    CardView {
        card_id: Some(CardIdentifier {
            side: adapters::player_side(source_side),
            index: target.index as u32,
            game_action: Some(CustomCardIdentifier::StatusMarker as u32),
            ability_id: Some(marker.source.index.0 as u32),
        }),
        card_position: Some(positions::for_custom_card(
            positions::stacked_behind_card(card.id),
            CustomCardIdentifier::StatusMarker,
        )),
        prefab: CardPrefab::TokenCard.into(),
        card_back: Some(assets::card_back(School::Neutral)),
        revealed_to_viewer: true,
        is_face_up: true,
        card_icons: None,
        arena_frame: Some(assets::arena_frame(
            source_side,
            source_definition.card_type,
            source_definition.config.resonance,
        )),
        face_down_arena_frame: None,
        owning_player: builder.to_player_name(source_side),
        revealed_card: Some(Box::new(RevealedCardView {
            card_frame: Some(assets::card_frame(source_definition.school, false)),
            title_background: Some(assets::ability_title_background()),
            jewel: Some(assets::jewel(source_definition.rarity)),
            image: Some(adapters::sprite(&source_definition.image)),
            image_background: None,
            title: Some(CardTitle {
                text: source_definition.name.displayed_name(),
                text_color: Some(assets::title_color(source_definition.config.resonance)),
            }),
            rules_text: Some(RulesText {
                text: rules_text::status_marker_text(
                    &CardViewContext::Game(source_definition, game, card),
                    marker,
                ),
            }),
            targeting: None,
            on_release_position: None,
            supplemental_info: None,
            card_move_target: None,
            point_to_parent: None,
            info_zoom_highlight: None,
        })),
        create_position: None,
        destroy_position: None,
        effects: None,
    }
}
