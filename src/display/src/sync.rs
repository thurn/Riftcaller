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

//! Converts a GameState into GameView updates

use adapters::response_builder::ResponseBuilder;
use constants::game_constants;
use game_data::card_state::{CardPositionKind, CardState};
use game_data::card_view_context::CardViewContext;
use game_data::character_preset::CharacterPreset;
use game_data::game_state::GameState;
use game_data::primitives::{RoomId, School, Side};
use protos::spelldawn::{
    ActionTrackerView, DeckView, GameView, ManaView, PlayerInfo, PlayerView, ScoreView,
};
use rules::mana::ManaPurpose;
use rules::{flags, mana, queries};
use {adapters, assets};

use crate::{button_prompt, card_sync, interface, positions, status_markers, tutorial_display};

pub fn run(builder: &mut ResponseBuilder, game: &GameState) {
    let cards = game
        .all_cards()
        .filter(|c| !skip_sending_to_client(c))
        .flat_map(|c| {
            let mut cards = vec![card_sync::card_view(
                builder,
                &CardViewContext::Game(rules::get(c.variant), game, c),
            )];
            cards.append(&mut card_sync::activated_ability_cards(builder, game, c));
            cards.append(&mut status_markers::build(builder, game, c));
            cards
        })
        .chain(
            (0..game.champion.curses)
                .map(|number| card_sync::curse_card_view(builder, Some(game), number)),
        )
        .chain((game.champion.curses > 0).then(|| card_sync::dispel_card_view(builder, Some(game))))
        .chain(
            (game.champion.wounds > 0)
                .then(|| card_sync::wound_card_view(builder, game.champion.wounds)),
        )
        .collect::<Vec<_>>();

    builder.push_game_view(GameView {
        user: Some(player_view(game, builder.user_side)),
        opponent: Some(player_view(game, builder.user_side.opponent())),
        cards,
        raid_active: game.raid.is_some(),
        game_object_positions: positions::game_object_positions(builder, game),
        main_controls: if builder.state.is_final_update {
            // Only include controls on final update to ensure interface doesn't show
            // previous UI after click.
            interface::render(builder, game)
        } else {
            None
        },
        tutorial_effects: if builder.state.is_final_update {
            // Likewise hide tutorial updates while animating
            tutorial_display::render(builder, &game.info.tutorial_state)
                .chain(button_prompt::append_prompt_speech_bubble(builder, game))
                .collect()
        } else {
            vec![]
        },
    });
}

fn player_view(game: &GameState, side: Side) -> PlayerView {
    PlayerView {
        side: adapters::player_side(side),
        player_info: Some(PlayerInfo {
            valid_rooms_to_visit: match side {
                Side::Overlord => enum_iterator::all::<RoomId>()
                    .filter(|room_id| flags::can_take_progress_action(game, side, *room_id))
                    .map(adapters::room_identifier)
                    .collect(),
                Side::Champion => enum_iterator::all::<RoomId>()
                    .filter(|room_id| flags::can_take_initiate_raid_action(game, side, *room_id))
                    .map(adapters::room_identifier)
                    .collect(),
            },
            appearance: Some(assets::character_preset(match side {
                Side::Overlord => CharacterPreset::Overlord,
                Side::Champion => CharacterPreset::Champion,
            })),
        }),
        score: Some(ScoreView { score: queries::score(game, side) }),
        mana: Some(ManaView {
            base_mana: mana::get(game, side, ManaPurpose::BaseMana),
            bonus_mana: mana::get(game, side, ManaPurpose::BonusForDisplay),
            can_take_gain_mana_action: flags::can_take_gain_mana_action(game, side),
        }),
        action_tracker: Some(ActionTrackerView {
            available_action_count: game.player(side).actions,
            default_action_count: match side {
                Side::Overlord => game_constants::OVERLORD_START_OF_TURN_ACTIONS,
                Side::Champion => game_constants::CHAMPION_START_OF_TURN_ACTIONS,
            },
        }),
        deck_view: Some(DeckView {
            card_back: Some(assets::card_back(
                *game.player(side).schools.get(0).unwrap_or(&School::Neutral),
            )),
            card_count: game.deck(side).count() as u32,
            can_take_draw_card_action: flags::can_take_draw_card_action(game, side),
        }),
        can_take_action: flags::has_priority(game, side),
    }
}

pub fn skip_sending_to_client(card: &CardState) -> bool {
    let hidden = !(card.is_visible_to(Side::Champion) || card.is_visible_to(Side::Overlord));
    let position = card.position().kind();
    position == CardPositionKind::DeckUnknown || (position == CardPositionKind::DeckTop && hidden)
}
