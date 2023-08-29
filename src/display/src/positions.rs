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

use adapters;
use adapters::response_builder::ResponseBuilder;
use anyhow::Result;
use game_data::card_state::{CardPosition, CardState};
use game_data::game::{GamePhase, GameState, MulliganData, RaidData};
use game_data::game_actions::{BrowserPromptTarget, CardTarget, GamePrompt, PromptContext};
use game_data::primitives::{
    AbilityId, CardId, GameObjectId, HasCardId, ItemLocation, RoomId, RoomLocation, Side,
};
use game_data::utils;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    ClientItemLocation, ClientRoomLocation, GameCharacterFacingDirection, GameObjectPositions,
    ObjectPosition, ObjectPositionBrowser, ObjectPositionBrowserDragTarget,
    ObjectPositionCardChoiceBrowser, ObjectPositionCharacter, ObjectPositionCharacterContainer,
    ObjectPositionDeck, ObjectPositionDeckContainer, ObjectPositionDiscardPile,
    ObjectPositionDiscardPileContainer, ObjectPositionHand, ObjectPositionIntoCard,
    ObjectPositionItem, ObjectPositionOffscreen, ObjectPositionRaid, ObjectPositionRevealedCards,
    ObjectPositionRiftcallers, ObjectPositionRoom, ObjectPositionStaging, RevealedCardsBrowserSize,
    RoomIdentifier,
};
use raids::traits::RaidDisplayState;
use raids::RaidDataExt;
use rules::queries;
use with_error::fail;

pub const RELEASE_SORTING_KEY: u32 = 100;

pub fn for_card(card: &CardState, position: Position) -> ObjectPosition {
    ObjectPosition {
        position: Some(position),
        sorting_key: 1 + card.sorting_key,
        sorting_subkey: 0,
    }
}

pub fn for_unveil_card(card: &CardState, position: Position) -> ObjectPosition {
    ObjectPosition {
        position: Some(position),
        sorting_key: 1 + card.sorting_key,
        sorting_subkey: 1,
    }
}

pub fn for_ability(game: &GameState, ability_id: AbilityId, position: Position) -> ObjectPosition {
    ObjectPosition {
        position: Some(position),
        sorting_key: 1 + game.card(ability_id.card_id).sorting_key,
        sorting_subkey: 2 + (ability_id.index.value() as u32),
    }
}

pub fn for_sorting_key(sorting_key: u32, position: Position) -> ObjectPosition {
    ObjectPosition { sorting_key: 1 + sorting_key, sorting_subkey: 0, position: Some(position) }
}

pub fn room(room_id: RoomId, location: RoomLocation) -> Position {
    Position::Room(ObjectPositionRoom {
        room_id: adapters::room_identifier(room_id),
        room_location: match location {
            RoomLocation::Defender => ClientRoomLocation::Front,
            RoomLocation::Occupant => ClientRoomLocation::Back,
        }
        .into(),
    })
}

pub fn unspecified_room(location: RoomLocation) -> Position {
    Position::Room(ObjectPositionRoom {
        room_id: RoomIdentifier::Unspecified as i32,
        room_location: match location {
            RoomLocation::Defender => ClientRoomLocation::Front,
            RoomLocation::Occupant => ClientRoomLocation::Back,
        }
        .into(),
    })
}

pub fn item(location: ItemLocation) -> Position {
    Position::Item(ObjectPositionItem {
        item_location: match location {
            ItemLocation::Weapons => ClientItemLocation::Left,
            ItemLocation::Artifacts => ClientItemLocation::Right,
        }
        .into(),
    })
}

pub fn hand(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Hand(ObjectPositionHand { owner: builder.to_player_name(side) })
}

pub fn deck(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Deck(ObjectPositionDeck { owner: builder.to_player_name(side) })
}

pub fn deck_container(builder: &ResponseBuilder, side: Side) -> Position {
    Position::DeckContainer(ObjectPositionDeckContainer { owner: builder.to_player_name(side) })
}

pub fn discard(builder: &ResponseBuilder, side: Side) -> Position {
    Position::DiscardPile(ObjectPositionDiscardPile { owner: builder.to_player_name(side) })
}

pub fn discard_container(builder: &ResponseBuilder, side: Side) -> Position {
    Position::DiscardPileContainer(ObjectPositionDiscardPileContainer {
        owner: builder.to_player_name(side),
    })
}

pub fn character(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Character(ObjectPositionCharacter { owner: builder.to_player_name(side) })
}

pub fn riftcaller(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Riftcaller(ObjectPositionRiftcallers { owner: builder.to_player_name(side) })
}

pub fn character_container(builder: &ResponseBuilder, side: Side) -> Position {
    Position::CharacterContainer(ObjectPositionCharacterContainer {
        owner: builder.to_player_name(side),
    })
}

pub fn staging() -> Position {
    Position::Staging(ObjectPositionStaging {})
}

pub fn accessed_browser() -> Position {
    Position::Browser(ObjectPositionBrowser {})
}

pub fn offscreen() -> Position {
    Position::Offscreen(ObjectPositionOffscreen {})
}

pub fn revealed_cards(large: bool) -> Position {
    Position::Revealed(ObjectPositionRevealedCards {
        size: if large { RevealedCardsBrowserSize::Large } else { RevealedCardsBrowserSize::Small }
            as i32,
    })
}

pub fn card_choice_browser() -> Position {
    Position::CardChoiceBrowser(ObjectPositionCardChoiceBrowser {})
}

pub fn raid() -> Position {
    Position::Raid(ObjectPositionRaid {})
}

pub fn parent_card(identifier: impl HasCardId) -> Position {
    Position::IntoCard(ObjectPositionIntoCard {
        card_id: Some(adapters::card_identifier(identifier.card_id())),
    })
}

/// The target position for the cards shown in a `CardBrowserPrompt`.
pub fn card_browser_target_position() -> Position {
    Position::BrowserDragTarget(ObjectPositionBrowserDragTarget {})
}

/// Calculates the game position in which the provided card should be displayed.
pub fn calculate(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Result<ObjectPosition> {
    Ok(if let Some(position_override) = position_override(builder, game, card)? {
        position_override
    } else {
        ObjectPosition {
            sorting_key: card.sorting_key,
            position: Some(adapt_position(builder, game, card.id, card.position())?),
            ..ObjectPosition::default()
        }
    })
}

fn adapt_position(
    builder: &ResponseBuilder,
    game: &GameState,
    card_id: CardId,
    position: CardPosition,
) -> Result<Position> {
    Ok(match position {
        CardPosition::Room(room_id, location) => room(room_id, location),
        CardPosition::ArenaItem(location) => item(location),
        CardPosition::Hand(side) => hand(builder, side),
        CardPosition::DeckTop(side) => deck(builder, side),
        CardPosition::DiscardPile(side) => discard(builder, side),
        CardPosition::Scored(side) => character(builder, side),
        CardPosition::Riftcaller(side) => riftcaller(builder, side),
        CardPosition::Scoring => staging(),
        CardPosition::Played(side, target) => {
            played_position(builder, game, side, card_id, target)?
        }
        CardPosition::DeckUnknown(_) => {
            fail!("Invalid card position")
        }
        CardPosition::GameModifier => offscreen(),
    })
}

/// Calculates the position of a card after it has been played.
///
/// For cards that are played by the opponent, we animate them to the staging
/// area. We also animate spell cards to staging while resolving their effects.
/// For other card types, we move them directly to their destination to make
/// playing a card feel more responsive.
pub fn played_position(
    builder: &ResponseBuilder,
    game: &GameState,
    side: Side,
    card_id: CardId,
    target: CardTarget,
) -> Result<Position> {
    if builder.user_side != side || rules::card_definition(game, card_id).card_type.is_spell() {
        Ok(staging())
    } else {
        adapt_position(
            builder,
            game,
            card_id,
            queries::played_position(game, side, card_id, target)?,
        )
    }
}

pub fn ability_card_position(
    builder: &ResponseBuilder,
    game: &GameState,
    ability_id: AbilityId,
) -> ObjectPosition {
    for_ability(
        game,
        ability_id,
        if utils::is_true(|| Some(game.ability_state.get(&ability_id)?.currently_resolving)) {
            staging()
        } else {
            hand(builder, ability_id.side())
        },
    )
}

pub fn game_object_positions(
    builder: &ResponseBuilder,
    game: &GameState,
) -> Result<GameObjectPositions> {
    let (side, opponent) = (builder.user_side, builder.user_side.opponent());
    Ok(GameObjectPositions {
        user_deck: Some(non_card(builder, game, GameObjectId::Deck(side))?),
        opponent_deck: Some(non_card(builder, game, GameObjectId::Deck(opponent))?),
        user_character: Some(non_card(builder, game, GameObjectId::Character(side))?),
        opponent_character: Some(non_card(builder, game, GameObjectId::Character(opponent))?),
        user_character_facing: character_facing_direction_for_side(
            builder,
            game,
            builder.user_side,
        )?
        .into(),
        opponent_character_facing: character_facing_direction_for_side(
            builder,
            game,
            builder.user_side.opponent(),
        )?
        .into(),
        user_discard: Some(non_card(builder, game, GameObjectId::DiscardPile(side))?),
        opponent_discard: Some(non_card(builder, game, GameObjectId::DiscardPile(opponent))?),
    })
}

fn non_card(
    builder: &ResponseBuilder,
    game: &GameState,
    id: GameObjectId,
) -> Result<ObjectPosition> {
    Ok(if let Some(position_override) = non_card_position_override(builder, game, id)? {
        position_override
    } else {
        match id {
            GameObjectId::Deck(side) => for_sorting_key(0, deck_container(builder, side)),
            GameObjectId::DiscardPile(side) => for_sorting_key(0, discard_container(builder, side)),
            GameObjectId::Character(side) => for_sorting_key(0, character_container(builder, side)),
            _ => fail!("Unsupported ID type"),
        }
    })
}

fn position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Result<Option<ObjectPosition>> {
    if let Some(o) = prompt_position_override(game, card) {
        return Ok(Some(o));
    }

    match &game.info.phase {
        GamePhase::ResolveMulligans(mulligans) => {
            Ok(opening_hand_position_override(builder, game, card, mulligans))
        }
        GamePhase::Play => raid_position_override(game, card.id.into()),
        _ => Ok(None),
    }
}

fn prompt_position_override(game: &GameState, card: &CardState) -> Option<ObjectPosition> {
    let current_prompt = game.player(card.side()).prompt_queue.get(0)?;

    match current_prompt {
        GamePrompt::ButtonPrompt(prompt) => {
            if prompt.context == Some(PromptContext::Card(card.id)) {
                return Some(for_card(card, accessed_browser()));
            }
            if prompt.choices.iter().any(|choice| choice.anchor_card == Some(card.id)) {
                return Some(for_card(card, card_choice_browser()));
            }
        }
        GamePrompt::CardBrowserPrompt(browser) => {
            if browser.unchosen_subjects.contains(&card.id) {
                return Some(for_card(card, revealed_cards(true)));
            } else if browser.chosen_subjects.contains(&card.id) {
                return Some(for_card(card, card_browser_target_position()));
            }
        }
    }

    None
}

fn non_card_position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    id: GameObjectId,
) -> Result<Option<ObjectPosition>> {
    let current_prompt = game.player(builder.user_side).prompt_queue.get(0);
    if let Some(GamePrompt::CardBrowserPrompt(browser)) = current_prompt {
        let target = match browser.target {
            BrowserPromptTarget::DiscardPile => GameObjectId::DiscardPile(builder.user_side),
            BrowserPromptTarget::Deck => GameObjectId::Deck(builder.user_side),
        };
        if id == target {
            return Ok(Some(for_sorting_key(
                0,
                Position::BrowserDragTarget(ObjectPositionBrowserDragTarget {}),
            )));
        }
    }

    raid_position_override(game, id)
}

fn raid_position_override(game: &GameState, id: GameObjectId) -> Result<Option<ObjectPosition>> {
    Ok(if let Some(raid_data) = &game.info.raid {
        match raid_data.phase().display_state(game)? {
            RaidDisplayState::None => None,
            RaidDisplayState::Defenders(defenders) => {
                browser_position(id, raid(), raid_browser(game, raid_data, defenders))
            }
            RaidDisplayState::Access => {
                browser_position(id, accessed_browser(), raid_access_browser(game, raid_data))
            }
        }
    } else {
        None
    })
}

fn opening_hand_position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
    data: &MulliganData,
) -> Option<ObjectPosition> {
    if data.decision(builder.user_side).is_none()
        && game.hand(builder.user_side).any(|c| c.id == card.id)
    {
        Some(for_card(card, revealed_cards(true)))
    } else {
        None
    }
}

fn browser_position(
    id: GameObjectId,
    position: Position,
    browser: Vec<GameObjectId>,
) -> Option<ObjectPosition> {
    browser.iter().position(|gid| *gid == id).map(|index| ObjectPosition {
        sorting_key: index as u32,
        sorting_subkey: 0,
        position: Some(position),
    })
}

fn raid_browser(game: &GameState, raid: &RaidData, defenders: Vec<CardId>) -> Vec<GameObjectId> {
    let mut result = Vec::new();

    match raid.target {
        RoomId::Vault => {
            result.push(GameObjectId::Deck(Side::Overlord));
        }
        RoomId::Sanctum => {
            result.push(GameObjectId::Character(Side::Overlord));
        }
        RoomId::Crypts => {
            result.push(GameObjectId::DiscardPile(Side::Overlord));
        }
        _ => {}
    }

    result.extend(game.occupants(raid.target).map(|card| GameObjectId::CardId(card.id)));
    result.extend(defenders.iter().map(|card_id| GameObjectId::CardId(*card_id)));
    result.push(GameObjectId::Character(Side::Champion));
    result
}

fn raid_access_browser(game: &GameState, raid: &RaidData) -> Vec<GameObjectId> {
    match raid.target {
        RoomId::Sanctum => {
            game.hand(Side::Overlord).map(|card| GameObjectId::CardId(card.id)).collect()
        }
        RoomId::Crypts => {
            game.discard_pile(Side::Overlord).map(|card| GameObjectId::CardId(card.id)).collect()
        }
        _ => raid.accessed.iter().map(|card_id| GameObjectId::CardId(*card_id)).collect(),
    }
}

fn character_facing_direction_for_side(
    builder: &ResponseBuilder,
    game: &GameState,
    side: Side,
) -> Result<GameCharacterFacingDirection> {
    if let Some(raid) = &game.info.raid {
        if matches!(raid.phase().display_state(game)?, RaidDisplayState::Defenders(_)) {
            if side == Side::Champion {
                return Ok(GameCharacterFacingDirection::Right);
            }

            if side == Side::Overlord && raid.target == RoomId::Sanctum {
                return Ok(GameCharacterFacingDirection::Left);
            }
        }
    }

    Ok(if builder.user_side == side {
        GameCharacterFacingDirection::Up
    } else {
        GameCharacterFacingDirection::Down
    })
}
