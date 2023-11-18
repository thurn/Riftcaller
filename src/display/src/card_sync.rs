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

use std::collections::HashSet;

use adapters::response_builder::ResponseBuilder;
use adapters::CustomCardIdentifier;
use constants::game_constants;
use core_ui::{design, icons};
use game_data::card_definition::{AbilityType, TargetRequirement};
use game_data::card_state::{CardState, OnPlayState};
use game_data::card_view_context::CardViewContext;
use game_data::continuous_visual_effect::ContinuousDisplayEffect;
use game_data::delegate_data::ContinuousDisplayEffectsQuery;
use game_data::game_actions::{CardTarget, GamePrompt};
use game_data::game_state::GameState;
use game_data::primitives::{
    AbilityId, CardId, CardType, ItemLocation, ManaValue, Rarity, RoomId, RoomLocation, School,
    Side, WoundCount,
};
use game_data::special_effects::TimedEffect;
use protos::spelldawn::card_targeting::Targeting;
use protos::spelldawn::{
    info_zoom_highlight, ArrowTargetRoom, CardEffects, CardIcons, CardPrefab, CardTargeting,
    CardTitle, CardView, EffectAddress, FlexColor, InfoZoomHighlight, NoTargeting, PlayInRoom,
    RevealedCardView, RulesText, TargetingArrow,
};
use rules::{dispatch, flags, queries};
use rules_text::{card_icons, supplemental_info};
use {adapters, assets, rules_text};

use crate::{card_selector, positions};

pub fn card_view(builder: &ResponseBuilder, context: &CardViewContext) -> CardView {
    let revealed = context.query_or(true, |_, card| card.is_visible_to(builder.user_side));
    CardView {
        card_id: context.query_or_none(|_, card| adapters::card_identifier(card.id)),
        card_position: context
            .query_or(None, |game, card| Some(positions::calculate(builder, game, card))),
        prefab: if context.definition().config.metadata.is_upgraded {
            CardPrefab::FullHeight
        } else {
            CardPrefab::Standard
        }
        .into(),
        card_back: Some(assets::card_back(
            context.query_or(context.definition().school, |game, card| {
                *game.player(card.side()).schools.get(0).unwrap_or(&School::Neutral)
            }),
        )),
        revealed_to_viewer: revealed,
        is_face_up: context.query_or(true, |_, card| card.is_face_up()),
        card_icons: Some(card_icons::build(context, revealed)),
        arena_frame: Some(assets::arena_frame(
            context.definition().side,
            context.definition().card_type,
            context.query_id_or(context.definition().config.resonance, queries::resonance),
        )),
        face_down_arena_frame: Some(assets::face_down_arena_frame()),
        owning_player: builder.to_player_name(context.definition().side),
        revealed_card: revealed.then(|| revealed_card_view(builder, context)),
        create_position: if builder.state.animate {
            context.query_or_none(|_, card| {
                positions::for_card(card, positions::deck(builder, card.side()))
            })
        } else {
            None
        },
        destroy_position: context.query_or_none(|_, card| {
            positions::for_card(card, positions::deck(builder, card.side()))
        }),
        effects: Some(CardEffects {
            outline_color: outline_color(context),
            arena_effect: continuous_display_effects(context),
        }),
    }
}

pub fn activated_ability_cards(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Vec<CardView> {
    let mut result = vec![];
    if card.side() != builder.user_side {
        return result;
    }

    let definition = rules::get(card.variant);

    if card.is_face_down() && card.position().in_play() {
        if builder.user_side == Side::Overlord && definition.card_type == CardType::Project {
            result.push(summon_project_card_view(builder, game, card.id));
        }

        return result;
    }

    for (ability_index, ability) in definition.abilities.iter().enumerate() {
        if let AbilityType::Activated { target_requirement, .. } = &ability.ability_type {
            let ability_id = AbilityId::new(card.id, ability_index);
            if card.position().in_play()
                || flags::activated_ability_has_valid_targets(game, builder.user_side, ability_id)
            {
                // We show all abilities of cards in play, but otherwise only show them
                // when they can be activated.
                result.push(ability_card_view(
                    builder,
                    game,
                    ability_id,
                    card.variant.metadata.full_art,
                    Some(target_requirement),
                ));
            }
        }
    }
    result
}

pub fn ability_card_view(
    builder: &ResponseBuilder,
    game: &GameState,
    ability_id: AbilityId,
    full_height: bool,
    target_requirement: Option<&TargetRequirement<AbilityId>>,
) -> CardView {
    let card = game.card(ability_id.card_id);
    let definition = rules::get(card.variant);
    let context = CardViewContext::Game(definition, game, card);

    CardView {
        card_id: Some(adapters::ability_card_identifier(ability_id)),
        card_position: Some(positions::ability_card_position(builder, game, ability_id)),
        prefab: if full_height { CardPrefab::FullHeightToken } else { CardPrefab::TokenCard }
            .into(),
        card_back: Some(assets::card_back(context.definition().school)),
        revealed_to_viewer: true,
        is_face_up: false,
        card_icons: Some(CardIcons {
            top_left_icon: queries::ability_mana_cost(game, ability_id)
                .map(card_icons::mana_card_icon),
            ..CardIcons::default()
        }),
        arena_frame: None,
        face_down_arena_frame: None,
        owning_player: builder.to_player_name(ability_id.card_id.side),
        revealed_card: Some(revealed_ability_card_view(
            &context,
            ability_id,
            full_height,
            target_requirement,
        )),
        create_position: if builder.state.animate {
            Some(positions::for_ability(game, ability_id, positions::parent_card(ability_id)))
        } else {
            None
        },
        destroy_position: Some(positions::for_ability(
            game,
            ability_id,
            positions::parent_card(ability_id),
        )),
        effects: None,
    }
}

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

fn revealed_card_view(
    builder: &ResponseBuilder,
    context: &CardViewContext,
) -> Box<RevealedCardView> {
    let definition = context.definition();
    let resonance = context.query_id_or(definition.config.resonance, queries::resonance);

    Box::new(RevealedCardView {
        card_frame: Some(assets::card_frame(
            definition.school,
            definition.config.metadata.full_art,
        )),
        title_background: Some(assets::title_background(resonance)),
        jewel: Some(assets::jewel(definition.rarity)),
        image: Some(adapters::sprite(&definition.image)),
        image_background: definition.config.image_background.as_ref().map(adapters::sprite),
        title: Some(CardTitle {
            text: definition.name.displayed_name(),
            text_color: Some(assets::title_color(resonance)),
        }),
        rules_text: Some(rules_text::build(context)),
        targeting: context.query_or_none(|game, card| {
            card_targeting(
                definition.config.custom_targeting.as_ref(),
                flags::enters_play_in_room(game, card.id),
                |target| flags::can_take_play_card_action(game, builder.user_side, card.id, target),
            )
        }),
        on_release_position: Some(positions::for_sorting_key(
            positions::RELEASE_SORTING_KEY,
            match definition.card_type {
                CardType::Artifact => positions::item(ItemLocation::Artifacts),
                CardType::Evocation => positions::item(ItemLocation::Evocations),
                CardType::Ally => positions::item(ItemLocation::Allies),
                CardType::OverlordSpell => positions::staging(),
                CardType::ChampionSpell => positions::staging(),
                CardType::Minion => positions::unspecified_room(RoomLocation::Defender),
                CardType::Project => positions::unspecified_room(RoomLocation::Occupant),
                CardType::Scheme => positions::unspecified_room(RoomLocation::Occupant),
                CardType::Riftcaller => positions::character_container(builder, definition.side),
                CardType::GameModifier => positions::offscreen(),
            },
        )),
        supplemental_info: supplemental_info::build(context, None),
        card_move_target: context
            .query_or(None, |game, card| card_selector::move_target(builder, game, card)),
        point_to_parent: None,
        info_zoom_highlight: context.card().and_then(info_zoom_highlight),
    })
}

fn info_zoom_highlight(card: &CardState) -> Option<InfoZoomHighlight> {
    match card.on_play_state() {
        OnPlayState::Card(card_id) => Some(InfoZoomHighlight {
            highlight: Some(info_zoom_highlight::Highlight::Card(adapters::card_identifier(
                *card_id,
            ))),
        }),
        OnPlayState::Room(room_id) => Some(InfoZoomHighlight {
            highlight: Some(info_zoom_highlight::Highlight::Room(adapters::room_identifier(
                *room_id,
            ))),
        }),
        _ => None,
    }
}

fn revealed_ability_card_view(
    context: &CardViewContext,
    ability_id: AbilityId,
    full_height: bool,
    target_requirement: Option<&TargetRequirement<AbilityId>>,
) -> Box<RevealedCardView> {
    let definition = context.definition();
    let ability = definition.ability(ability_id.index);
    Box::new(RevealedCardView {
        card_frame: Some(assets::card_frame(definition.school, full_height)),
        title_background: Some(assets::ability_title_background()),
        jewel: Some(assets::jewel(definition.rarity)),
        image: Some(adapters::sprite(&definition.image)),
        image_background: definition.config.image_background.as_ref().map(adapters::sprite),
        title: Some(CardTitle {
            text: format!(
                "{} {} {}",
                icons::BULLET,
                definition.name.displayed_name(),
                icons::BULLET
            ),
            text_color: Some(assets::title_color(None)),
        }),
        rules_text: Some(RulesText { text: rules_text::ability_text(context, ability) }),
        targeting: context.query_or_none(|game, _| {
            card_targeting(target_requirement, false, |target| {
                flags::can_take_activate_ability_action(game, ability_id.side(), ability_id, target)
            })
        }),
        on_release_position: context.query_or_none(|game, _| {
            positions::for_ability(game, ability_id, positions::staging())
        }),
        supplemental_info: supplemental_info::build(context, Some(ability_id.index)),
        card_move_target: None,
        point_to_parent: Some(adapters::card_identifier(ability_id.card_id)),
        info_zoom_highlight: None,
    })
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
    status_card_view(
        builder,
        Side::Champion,
        StatusCard {
            identifier: CustomCardIdentifier::SummonProject,
            counters: count,
            image: "wound".to_string(),
            title: "Wound".to_string(),
            text: "-1 maximum hand size".to_string(),
        },
    )
}

pub struct StatusCard {
    pub identifier: CustomCardIdentifier,
    pub counters: u32,
    pub image: String,
    pub title: String,
    pub text: String,
}

fn status_card_view(builder: &ResponseBuilder, side: Side, card: StatusCard) -> CardView {
    CardView {
        card_id: Some(adapters::custom_card_identifier(card.identifier, 0)),
        card_position: Some(positions::for_custom_card(
            positions::riftcaller(builder, side),
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

fn boolean_target(can_play: impl Fn(CardTarget) -> bool) -> CardTargeting {
    let no_target: Option<&TargetRequirement<()>> = None;
    card_targeting(no_target, false, can_play)
}

fn card_targeting<T>(
    requirement: Option<&TargetRequirement<T>>,
    play_in_room: bool,
    can_play: impl Fn(CardTarget) -> bool,
) -> CardTargeting {
    CardTargeting {
        targeting: Some(match (requirement, play_in_room) {
            (None, false) | (Some(TargetRequirement::None), _) => {
                Targeting::NoTargeting(NoTargeting { can_play: can_play(CardTarget::None) })
            }
            (None, true) | (Some(TargetRequirement::TargetRoom(_)), _) => {
                let valid = enum_iterator::all::<RoomId>()
                    .filter(|room_id| can_play(CardTarget::Room(*room_id)))
                    .map(adapters::room_identifier)
                    .collect();

                if play_in_room {
                    Targeting::PlayInRoom(PlayInRoom { valid_rooms: valid })
                } else {
                    Targeting::ArrowTargetRoom(ArrowTargetRoom {
                        valid_rooms: valid,
                        arrow: TargetingArrow::Red.into(),
                    })
                }
            }
        }),
    }
}

fn outline_color(context: &CardViewContext) -> Option<FlexColor> {
    if let CardViewContext::Game(_, game, card) = context {
        if let Some(GamePrompt::PlayCardBrowser(browser)) =
            &game.player(card.side()).prompt_queue.get(0)
        {
            if browser.cards.contains(&card.id) {
                return Some(design::PLAY_CARD_BROWSER_OUTLINE);
            }
        }
    }

    None
}

fn continuous_display_effects(context: &CardViewContext) -> Option<EffectAddress> {
    let CardViewContext::Game(_, game, card) = context else {
        return None;
    };

    let effects =
        dispatch::perform_query(game, ContinuousDisplayEffectsQuery(card.id), HashSet::new());

    // Effects are not ordered so we just pick an arbitrary element from the set to
    // display.
    if let Some(ContinuousDisplayEffect::CannotSummonThisTurn) = effects.iter().next() {
        return Some(assets::timed_effect(TimedEffect::MagicCircles1Looping(
            "Magic circle 2 loop",
        )));
    }

    None
}
