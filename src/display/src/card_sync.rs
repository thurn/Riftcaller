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

use adapters::response_builder::ResponseBuilder;
use anyhow::Result;
use core_ui::icons;
use data::card_definition::{AbilityType, TargetRequirement};
use data::card_state::CardState;
use data::game::GameState;
use data::game_actions::CardTarget;
use data::primitives::{AbilityId, CardType, ItemLocation, RoomId, RoomLocation};
use data::text::RulesTextContext;
use protos::spelldawn::card_targeting::Targeting;
use protos::spelldawn::{
    ArrowTargetRoom, CardIcons, CardPrefab, CardTargeting, CardTitle, CardView, NoTargeting,
    PlayInRoom, RevealedCardView, RulesText, SpriteAddress, TargetingArrow,
};
use rules::{flags, queries};
use rules_text::card_icons;
use {adapters, assets, rules_text};

use crate::positions;

pub fn card_view(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Result<CardView> {
    let definition = rules::get(card.name);
    let revealed = card.is_revealed_to(builder.user_side);
    Ok(CardView {
        card_id: Some(adapters::card_identifier(card.id)),
        card_position: Some(positions::convert(builder, game, card)?),
        prefab: if definition.card_type == CardType::Leader {
            CardPrefab::FullHeight
        } else {
            CardPrefab::Standard
        }
        .into(),
        revealed_to_viewer: card.is_revealed_to(builder.user_side),
        is_face_up: card.is_face_up(),
        card_icons: Some(card_icons::build(
            &RulesTextContext::Game(game, card),
            definition,
            revealed,
        )),
        arena_frame: Some(assets::arena_frame(
            definition.side,
            definition.card_type,
            definition.config.lineage,
        )),
        face_down_arena_frame: Some(assets::face_down_arena_frame()),
        owning_player: builder.to_player_name(definition.side),
        revealed_card: revealed.then(|| revealed_card_view(builder, game, card)),
        create_position: if builder.state.animate {
            Some(positions::for_card(card, positions::deck(builder, card.side())))
        } else {
            None
        },
        destroy_position: Some(positions::for_card(card, positions::deck(builder, card.side()))),
    })
}

pub fn activated_ability_cards(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Vec<Result<CardView>> {
    let mut result = vec![];
    if card.side() != builder.user_side || !card.position().in_play() {
        return result;
    }

    for (ability_index, ability) in rules::get(card.name).abilities.iter().enumerate() {
        if let AbilityType::Activated(_, target_requirement) = &ability.ability_type {
            let ability_id = AbilityId::new(card.id, ability_index);
            result.push(Ok(ability_card_view(builder, game, ability_id, Some(target_requirement))));
        }
    }
    result
}

pub fn ability_card_view(
    builder: &ResponseBuilder,
    game: &GameState,
    ability_id: AbilityId,
    target_requirement: Option<&TargetRequirement<AbilityId>>,
) -> CardView {
    CardView {
        card_id: Some(adapters::ability_card_identifier(ability_id)),
        card_position: Some(positions::ability_card_position(builder, game, ability_id)),
        prefab: CardPrefab::TokenCard.into(),
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
            builder,
            game,
            ability_id,
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
    }
}

fn revealed_card_view(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> RevealedCardView {
    let definition = rules::get(card.name);
    RevealedCardView {
        card_frame: Some(assets::card_frame(definition.school, definition.card_type)),
        title_background: Some(assets::title_background(definition.config.lineage)),
        jewel: Some(assets::jewel(definition.rarity)),
        image: Some(adapters::sprite(&definition.image)),
        image_background: definition.config.image_background.as_ref().map(adapters::sprite),
        title: Some(CardTitle {
            text: definition.name.displayed_name(),
            text_color: Some(assets::title_color(definition.config.lineage)),
        }),
        rules_text: Some(rules_text::build(&RulesTextContext::Game(game, card), definition)),
        targeting: Some(card_targeting(
            definition.config.custom_targeting.as_ref(),
            flags::enters_play_in_room(game, card.id),
            |target| flags::can_take_play_card_action(game, builder.user_side, card.id, target),
        )),
        on_release_position: Some(positions::for_sorting_key(
            positions::RELEASE_SORTING_KEY,
            match definition.card_type {
                CardType::Weapon => positions::item(ItemLocation::Weapons),
                CardType::Artifact => positions::item(ItemLocation::Artifacts),
                CardType::OverlordSpell => positions::staging(),
                CardType::ChampionSpell => positions::staging(),
                CardType::Minion => positions::unspecified_room(RoomLocation::Defender),
                CardType::Project => positions::unspecified_room(RoomLocation::Occupant),
                CardType::Scheme => positions::unspecified_room(RoomLocation::Occupant),
                CardType::Leader => positions::staging(),
            },
        )),
        supplemental_info: rules_text::build_supplemental_info(
            &RulesTextContext::Game(game, card),
            None,
        ),
    }
}

fn revealed_ability_card_view(
    _builder: &ResponseBuilder,
    game: &GameState,
    ability_id: AbilityId,
    target_requirement: Option<&TargetRequirement<AbilityId>>,
) -> RevealedCardView {
    let card = game.card(ability_id.card_id);
    let definition = rules::get(card.name);
    let ability = definition.ability(ability_id.index);
    RevealedCardView {
        card_frame: Some(assets::card_frame(definition.school, definition.card_type)),
        title_background: Some(SpriteAddress {
            address: "LittleSweetDaemon/TCG_Card_Design/Custom/Title/TokenTitleBackground.png"
                .to_string(),
        }),
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
        rules_text: Some(RulesText {
            text: rules_text::ability_text(&RulesTextContext::Game(game, card), ability),
        }),
        targeting: Some(card_targeting(target_requirement, false, |target| {
            flags::can_take_activate_ability_action(game, ability_id.side(), ability_id, target)
        })),
        on_release_position: Some(positions::for_ability(game, ability_id, positions::staging())),
        supplemental_info: rules_text::build_supplemental_info(
            &RulesTextContext::Game(game, card),
            Some(ability_id.index),
        ),
    }
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
