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
use constants::game_constants;
use core_data::game_primitives::{CardId, CardType, ManaValue, Rarity, School, Side, WoundCount};
use core_ui::icons;
use game_data::card_definition::TargetRequirement;
use game_data::card_view_context::CardViewContext;
use game_data::game_actions::CardTarget;
use game_data::game_state::GameState;
use game_data::prompt_data::{GamePrompt, RoomSelectorPrompt};
use protos::riftcaller::{
    card_targeting, ArrowTargetRoom, CardIcons, CardPrefab, CardTargeting, CardTitle, CardView,
    RevealedCardView, RulesText, TargetingArrow,
};
use rules::{flags, queries, CardDefinitionExt};
use rules_text::{card_icons, supplemental_info};

use crate::{card_sync, positions};

pub fn summon_project_card_view(
    builder: &ResponseBuilder,
    game: &GameState,
    card_id: CardId,
) -> CardView {
    let card = game.card(card_id);
    let definition = rules::get(card.variant);
    let context = CardViewContext::Game(definition, game, card);

    CardView {
        card_id: Some(adapters::summon_project_card_identifier(card_id)),
        card_position: Some(positions::for_summon_project_card(
            card,
            positions::hand(builder, card_id.side),
        )),
        prefab: CardPrefab::TokenCard.into(),
        card_back: Some(assets::card_back(context.definition().school)),
        revealed_to_viewer: true,
        is_face_up: false,
        card_icons: Some(CardIcons {
            top_left_icon: queries::mana_cost(game, card_id).map(card_icons::mana_card_icon),
            ..CardIcons::default()
        }),
        arena_frame: None,
        face_down_arena_frame: None,
        owning_player: builder.to_player_name(card_id.side),
        revealed_card: Some(revealed_summon_project_card_view(&context, card_id)),
        create_position: if builder.state.animate {
            Some(positions::for_summon_project_card(card, positions::parent_card(card_id)))
        } else {
            None
        },
        destroy_position: Some(positions::for_summon_project_card(
            card,
            positions::parent_card(card_id),
        )),
        effects: None,
    }
}

fn revealed_summon_project_card_view(
    context: &CardViewContext,
    card_id: CardId,
) -> Box<RevealedCardView> {
    let definition = context.definition();
    Box::new(RevealedCardView {
        card_frame: Some(assets::card_frame(definition.school, false)),
        title_background: Some(assets::ability_title_background()),
        jewel: Some(assets::jewel(definition.rarity)),
        image: Some(adapters::sprite(&definition.image)),
        image_background: definition.config.image_background.as_ref().map(adapters::sprite),
        title: Some(CardTitle {
            text: format!(
                "<size=75%>{}</size> {} <size=75%>{}</size>",
                icons::CARET_UP,
                definition.name.displayed_name(),
                icons::CARET_UP
            ),
            text_color: Some(assets::title_color(None)),
        }),
        rules_text: Some(rules_text::build(context)),
        targeting: context.query_or_none(|game, _| {
            boolean_target(|_| flags::can_take_summon_project_action(game, Side::Overlord, card_id))
        }),
        on_release_position: context.query_or_none(|_, card| {
            positions::for_summon_project_card(card, positions::parent_card(card_id))
        }),
        supplemental_info: supplemental_info::build(context, None),
        card_move_target: None,
        point_to_parent: Some(adapters::card_identifier(card_id)),
        info_zoom_highlight: None,
    })
}

pub fn curse_card_view(
    builder: &ResponseBuilder,
    game: Option<&GameState>,
    number: u32,
) -> CardView {
    action_card_view(
        builder,
        game,
        ActionCard {
            action: CustomCardIdentifier::Curse,
            identifier_number: number,
            cost: game_constants::COST_TO_REMOVE_CURSE,
            image: "curse".to_string(),
            title: "Curse".to_string(),
            text: format!(
                "While this is held, the Overlord may pay {},2{} to destroy an evocation.",
                icons::ACTION,
                icons::MANA
            ),
            side: Side::Champion,
            can_play_fn: flags::can_take_remove_curse_action,
        },
    )
}

pub fn dispel_card_view(builder: &ResponseBuilder, game: Option<&GameState>) -> CardView {
    action_card_view(
        builder,
        game,
        ActionCard {
            action: CustomCardIdentifier::Dispel,
            identifier_number: 1,
            cost: game_constants::COST_TO_DISPEL_EVOCATION,
            image: "dispel".to_string(),
            title: "Dispel Evocation".to_string(),
            text: "Play only if the Champion is cursed.\nDestroy target evocation.".to_string(),
            side: Side::Overlord,
            can_play_fn: flags::can_take_dispel_evocation_action,
        },
    )
}

pub struct ActionCard {
    pub action: CustomCardIdentifier,
    pub identifier_number: u32,
    pub cost: ManaValue,
    pub image: String,
    pub title: String,
    pub text: String,
    pub side: Side,
    pub can_play_fn: fn(&GameState, Side) -> bool,
}

fn action_card_view(
    builder: &ResponseBuilder,
    game: Option<&GameState>,
    card: ActionCard,
) -> CardView {
    let character_position = game
        .map(|_| positions::for_custom_card(positions::character(builder, card.side), card.action));
    CardView {
        card_id: Some(adapters::custom_card_identifier(card.action, card.identifier_number)),
        card_position: Some(positions::for_custom_card(
            positions::hand(builder, card.side),
            card.action,
        )),
        prefab: CardPrefab::TokenCard.into(),
        card_back: Some(assets::card_back(School::Neutral)),
        revealed_to_viewer: true,
        is_face_up: false,
        card_icons: Some(CardIcons {
            top_left_icon: Some(card_icons::mana_card_icon(card.cost)),
            ..CardIcons::default()
        }),
        arena_frame: None,
        face_down_arena_frame: None,
        owning_player: builder.to_player_name(card.side),
        revealed_card: Some(Box::new(RevealedCardView {
            card_frame: Some(assets::card_frame(School::Neutral, false)),
            title_background: Some(assets::ability_title_background()),
            jewel: Some(assets::jewel(Rarity::None)),
            image: Some(adapters::sprite(&assets::misc_card(card.image, false))),
            image_background: None,
            title: Some(CardTitle {
                text: card.title,
                text_color: Some(assets::title_color(None)),
            }),
            rules_text: Some(RulesText { text: card.text }),
            targeting: game.map(|g| boolean_target(|_| (card.can_play_fn)(g, builder.user_side))),
            on_release_position: character_position.clone(),
            supplemental_info: None,
            card_move_target: None,
            point_to_parent: None,
            info_zoom_highlight: None,
        })),
        create_position: if builder.state.animate { character_position.clone() } else { None },
        destroy_position: character_position,
        effects: None,
    }
}

pub fn wound_card_view(builder: &ResponseBuilder, count: WoundCount) -> CardView {
    counter_card_view(
        builder,
        Side::Champion,
        CounterCard {
            identifier: CustomCardIdentifier::Wound,
            counters: count,
            image: "wound".to_string(),
            title: "Wound".to_string(),
            text: "-1 maximum hand size.".to_string(),
        },
    )
}

pub fn leyline_card_view(builder: &ResponseBuilder, count: u32) -> CardView {
    counter_card_view(
        builder,
        Side::Champion,
        CounterCard {
            identifier: CustomCardIdentifier::Leyline,
            counters: count,
            image: "leyline".to_string(),
            title: "Leyline".to_string(),
            text: format!("Gain 1{} to use during each raid.", icons::MANA),
        },
    )
}

pub struct CounterCard {
    pub identifier: CustomCardIdentifier,
    pub counters: u32,
    pub image: String,
    pub title: String,
    pub text: String,
}

fn counter_card_view(builder: &ResponseBuilder, side: Side, card: CounterCard) -> CardView {
    CardView {
        card_id: Some(adapters::custom_card_identifier(card.identifier, 0)),
        card_position: Some(positions::for_custom_card(
            positions::display_shelf(builder, side),
            card.identifier,
        )),
        prefab: CardPrefab::TokenCard.into(),
        card_back: Some(assets::card_back(School::Neutral)),
        revealed_to_viewer: true,
        is_face_up: true,
        card_icons: if card.counters > 1 {
            Some(CardIcons {
                arena_icon: Some(card_icons::status_quantity_icon(card.counters)),
                ..CardIcons::default()
            })
        } else {
            None
        },
        arena_frame: Some(assets::arena_frame(Side::Champion, CardType::GameModifier, None)),
        face_down_arena_frame: None,
        owning_player: builder.to_player_name(side),
        revealed_card: Some(Box::new(RevealedCardView {
            card_frame: Some(assets::card_frame(School::Neutral, false)),
            title_background: Some(assets::ability_title_background()),
            jewel: Some(assets::jewel(Rarity::None)),
            image: Some(adapters::sprite(&assets::misc_card(card.image, false))),
            image_background: None,
            title: Some(CardTitle {
                text: card.title,
                text_color: Some(assets::title_color(None)),
            }),
            rules_text: Some(RulesText { text: card.text }),
            targeting: None,
            on_release_position: None,
            supplemental_info: None,
            card_move_target: None,
            point_to_parent: None,
            info_zoom_highlight: None,
        })),
        create_position: Some(positions::for_custom_card(
            positions::character(builder, side),
            card.identifier,
        )),
        destroy_position: Some(positions::for_custom_card(
            positions::character(builder, side),
            card.identifier,
        )),
        effects: None,
    }
}

pub fn room_selector_card_view(
    builder: &ResponseBuilder,
    game: &GameState,
) -> impl Iterator<Item = CardView> {
    let result = if let Some(GamePrompt::RoomSelector(prompt)) =
        game.player(builder.user_side).prompt_stack.current()
    {
        Some(room_selector(builder, game, prompt))
    } else {
        None
    };

    result.into_iter()
}

fn room_selector(
    builder: &ResponseBuilder,
    game: &GameState,
    prompt: &RoomSelectorPrompt,
) -> CardView {
    let definition = game.card(prompt.initiated_by.card_id).definition();
    let character_position = Some(positions::for_custom_card(
        positions::character(builder, definition.side),
        CustomCardIdentifier::RoomSelector,
    ));

    CardView {
        card_id: Some(adapters::custom_card_identifier(CustomCardIdentifier::RoomSelector, 1)),
        card_position: Some(positions::for_custom_card(
            positions::hand(builder, definition.side),
            CustomCardIdentifier::RoomSelector,
        )),
        prefab: CardPrefab::TokenCard.into(),
        card_back: Some(assets::card_back(definition.school)),
        revealed_to_viewer: true,
        is_face_up: false,
        card_icons: None,
        arena_frame: None,
        face_down_arena_frame: None,
        owning_player: builder.to_player_name(definition.side),
        revealed_card: Some(Box::new(RevealedCardView {
            card_frame: Some(assets::card_frame(definition.school, false)),
            title_background: Some(assets::ability_title_background()),
            jewel: Some(assets::jewel(Rarity::None)),
            image: Some(adapters::sprite(&definition.image)),
            image_background: None,
            title: Some(CardTitle {
                text: definition.name.displayed_name(),
                text_color: Some(assets::title_color(None)),
            }),
            rules_text: Some(RulesText { text: "Select target room".to_string() }),
            targeting: Some(CardTargeting {
                targeting: Some(card_targeting::Targeting::ArrowTargetRoom(ArrowTargetRoom {
                    valid_rooms: prompt
                        .valid_rooms
                        .iter()
                        .map(|r| adapters::room_identifier(*r))
                        .collect(),
                    arrow: TargetingArrow::Blue.into(),
                })),
            }),
            on_release_position: character_position.clone(),
            supplemental_info: None,
            card_move_target: None,
            point_to_parent: None,
            info_zoom_highlight: None,
        })),
        create_position: if builder.state.animate { character_position.clone() } else { None },
        destroy_position: character_position,
        effects: None,
    }
}

fn boolean_target(can_play: impl Fn(CardTarget) -> bool) -> CardTargeting {
    let no_target: Option<&TargetRequirement<()>> = None;
    card_sync::card_targeting(no_target, false, can_play)
}
