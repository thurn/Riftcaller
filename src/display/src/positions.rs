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

use adapters;
use adapters::response_builder::ResponseBuilder;
use adapters::CustomCardIdentifier;
use card_definition_data::cards::CardDefinitionExt;
use core_data::game_primitives::{
    AbilityId, CardId, CardPlayId, GameObjectId, HasCardId, ItemLocation, RoomId, RoomLocation,
    Side,
};
use game_data::card_state::{CardPosition, CardState};
use game_data::game_actions::{CardTarget, DisplayPreference};
use game_data::game_state::{GamePhase, GameState, MulliganData};
use game_data::prompt_data::{GamePrompt, SelectorPromptTarget};
use game_data::raid_data::{RaidData, RaidDisplayState};
use protos::riftcaller::object_position::Position;
use protos::riftcaller::{
    ClientItemLocation, ClientRoomLocation, GameCharacterFacingDirection, GameObjectPositions,
    ObjectPosition, ObjectPositionBrowser, ObjectPositionBrowserDragTarget,
    ObjectPositionCardChoiceBrowser, ObjectPositionCharacter, ObjectPositionCharacterContainer,
    ObjectPositionDeck, ObjectPositionDeckContainer, ObjectPositionDiscardPile,
    ObjectPositionDiscardPileContainer, ObjectPositionHand, ObjectPositionHandStorage,
    ObjectPositionIntoCard, ObjectPositionItem, ObjectPositionOffscreen, ObjectPositionRaid,
    ObjectPositionRevealedCards, ObjectPositionRiftcallers, ObjectPositionRoom,
    ObjectPositionScoring, ObjectPositionStackedBehindCard, ObjectPositionStaging,
    RevealedCardsBrowserSize, RoomIdentifier,
};
use raid_display::raid_display_state;
use rules::{activate_ability, prompts, queries};

pub const RELEASE_SORTING_KEY: u32 = 100;

pub fn for_card(card: &CardState, position: Position) -> ObjectPosition {
    ObjectPosition {
        position: Some(position),
        sorting_key: 1 + card.sorting_key,
        sorting_subkey: 0,
    }
}

pub fn for_summon_project_card(card: &CardState, position: Position) -> ObjectPosition {
    ObjectPosition {
        position: Some(position),
        sorting_key: 1 + card.sorting_key,
        sorting_subkey: 1,
    }
}

pub fn for_custom_card(position: Position, id: CustomCardIdentifier) -> ObjectPosition {
    ObjectPosition { position: Some(position), sorting_key: 100, sorting_subkey: 1 + id as u32 }
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
            ItemLocation::Artifacts => ClientItemLocation::Left,
            ItemLocation::Evocations => ClientItemLocation::Right,
            ItemLocation::Allies => ClientItemLocation::Right,
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

pub fn display_shelf(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Riftcaller(ObjectPositionRiftcallers { owner: builder.to_player_name(side) })
}

pub fn character_container(builder: &ResponseBuilder, side: Side) -> Position {
    Position::CharacterContainer(ObjectPositionCharacterContainer {
        owner: builder.to_player_name(side),
    })
}

pub fn scoring() -> Position {
    Position::Scoring(ObjectPositionScoring {})
}

pub fn staging() -> Position {
    Position::Staging(ObjectPositionStaging {})
}

pub fn card_browser() -> Position {
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

pub fn hand_storage() -> Position {
    Position::HandStorage(ObjectPositionHandStorage {})
}

pub fn raid() -> Position {
    Position::Raid(ObjectPositionRaid {})
}

pub fn parent_card(identifier: impl HasCardId) -> Position {
    Position::IntoCard(ObjectPositionIntoCard {
        card_id: Some(adapters::card_identifier(identifier.card_id())),
    })
}

pub fn stacked_behind_card(identifier: impl HasCardId) -> Position {
    Position::StackedBehindCard(ObjectPositionStackedBehindCard {
        card_id: Some(adapters::card_identifier(identifier.card_id())),
    })
}

/// The target position for the cards shown in a `CardBrowserPrompt`.
pub fn card_browser_target_position() -> Position {
    Position::BrowserDragTarget(ObjectPositionBrowserDragTarget {})
}

/// Returns true if the provided `card` has an active position override, e.g.
/// because it is being displayed in a raid browser or as part of a prompt.
pub fn has_position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> bool {
    position_override(builder, game, card).is_some()
}

/// Calculates the game position in which the provided card should be displayed.
pub fn calculate(builder: &ResponseBuilder, game: &GameState, card: &CardState) -> ObjectPosition {
    if let Some(position_override) = position_override(builder, game, card) {
        return position_override;
    }

    ObjectPosition {
        sorting_key: card.sorting_key,
        position: adapt_position(builder, game, card.id, Some(card.position())),
        ..ObjectPosition::default()
    }
}

fn adapt_position(
    builder: &ResponseBuilder,
    game: &GameState,
    card_id: CardId,
    position: Option<CardPosition>,
) -> Option<Position> {
    let Some(p) = position else {
        return None;
    };

    match p {
        CardPosition::Room(_, room_id, location) => Some(room(room_id, location)),
        CardPosition::ArenaItem(_, location) => Some(item(location)),
        CardPosition::Hand(side) => Some(hand(builder, side)),
        CardPosition::DeckTop(side) => Some(deck(builder, side)),
        CardPosition::DiscardPile(side) => Some(discard(builder, side)),
        CardPosition::Scored(side) => Some(character(builder, side)),
        CardPosition::Identity(side) => Some(display_shelf(builder, side)),
        CardPosition::Scoring => Some(scoring()),
        CardPosition::Played(card_play_id, side, target) => {
            played_position(builder, game, side, card_id, target, card_play_id)
        }
        CardPosition::DeckUnknown(..) => None,
        CardPosition::GameModifier => Some(offscreen()),
        CardPosition::Sigil(side) => Some(display_shelf(builder, side)),
        CardPosition::Banished(Some(by_card))
            if game.card(by_card.source).last_card_play_id == Some(by_card.play_id) =>
        {
            // If this card is currently banished by the banish event of another card,
            // render it stacked underneath that card.
            Some(stacked_behind_card(by_card.source))
        }
        CardPosition::Banished(..) => Some(offscreen()),
    }
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
    card_play_id: CardPlayId,
) -> Option<Position> {
    if builder.user_side != side || game.card(card_id).definition().card_type.is_spell() {
        Some(staging())
    } else {
        adapt_position(
            builder,
            game,
            card_id,
            queries::played_position(game, side, card_id, target, card_play_id),
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
        if activate_ability::is_current_ability(game, ability_id) {
            staging()
        } else {
            hand(builder, ability_id.side())
        },
    )
}

pub fn game_object_positions(
    builder: &ResponseBuilder,
    game: &GameState,
) -> Option<GameObjectPositions> {
    let (side, opponent) = (builder.user_side, builder.user_side.opponent());
    Some(GameObjectPositions {
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
) -> Option<ObjectPosition> {
    if let Some(position_override) = non_card_position_override(builder, game, id) {
        if !disable_position_overrides(builder, game) {
            return Some(position_override);
        }
    }

    match id {
        GameObjectId::Deck(side) => Some(for_sorting_key(0, deck_container(builder, side))),
        GameObjectId::DiscardPile(side) => {
            Some(for_sorting_key(0, discard_container(builder, side)))
        }
        GameObjectId::Character(side) => {
            Some(for_sorting_key(0, character_container(builder, side)))
        }
        _ => None,
    }
}

fn disable_position_overrides(builder: &ResponseBuilder, game: &GameState) -> bool {
    builder.state.display_preference == Some(DisplayPreference::ShowArenaView(true))
        || matches!(
            prompts::current(game, builder.user_side),
            Some(GamePrompt::PlayCardBrowser(..)) | Some(GamePrompt::RoomSelector(..))
        )
}

fn position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Option<ObjectPosition> {
    if let Some(o) = prompt_position_override(builder, game, card) {
        return Some(o);
    }

    if disable_position_overrides(builder, game) {
        return None;
    }

    if let Some(o) = opponent_prompt_position_override(builder, game, card) {
        return Some(o);
    }

    match &game.info.phase {
        GamePhase::ResolveMulligans(mulligans) => {
            opening_hand_position_override(builder, game, card, mulligans)
        }
        GamePhase::Play => raid_position_override(game, card.id.into()),
        _ => None,
    }
}

fn prompt_position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Option<ObjectPosition> {
    let current_prompt = prompts::current(game, builder.user_side)?;

    match current_prompt {
        GamePrompt::ButtonPrompt(prompt) => {
            if disable_position_overrides(builder, game) {
                return None;
            }

            if prompt.context.as_ref().and_then(|c| c.associated_card()) == Some(card.id) {
                return Some(for_card(card, card_browser()));
            }
            if prompt.choices.iter().any(|choice| choice.anchor_card == Some(card.id)) {
                return Some(for_card(card, card_choice_browser()));
            }
        }
        GamePrompt::CardSelector(browser) => {
            if disable_position_overrides(builder, game) {
                return None;
            }

            if let Some(i) = browser.unchosen_subjects.iter().position(|&id| id == card.id) {
                // The position we set in [non_card_position_override] for the target is at
                // sorting key 1, so this needs to be at least 2. I don't remember why I did
                // this but it was probably important.
                return Some(ObjectPosition {
                    sorting_key: 10 + i as u32,
                    sorting_subkey: 0,
                    position: Some(card_browser()),
                });
            } else if let Some(i) = browser.chosen_subjects.iter().position(|&id| id == card.id) {
                return Some(ObjectPosition {
                    sorting_key: 10 + i as u32,
                    sorting_subkey: 0,
                    position: Some(card_browser_target_position()),
                });
            } else if card.position() == CardPosition::Hand(builder.user_side) {
                return Some(for_card(card, hand_storage()));
            }
        }
        GamePrompt::PlayCardBrowser(play_card) => {
            if play_card.cards.contains(&card.id) {
                return Some(for_card(card, hand(builder, card.side())));
            } else if card.position() == CardPosition::Hand(builder.user_side) {
                return Some(for_card(card, hand_storage()));
            }
        }
        GamePrompt::PriorityPrompt => {}
        GamePrompt::RoomSelector(..) => {
            if card.position() == CardPosition::Hand(builder.user_side) {
                return Some(for_card(card, hand_storage()));
            }
        }
    }

    None
}

fn opponent_prompt_position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Option<ObjectPosition> {
    let current_prompt = prompts::current(game, builder.user_side.opponent())?;
    if let GamePrompt::ButtonPrompt(prompt) = current_prompt {
        if prompt.context.as_ref().and_then(|c| c.associated_card()) == Some(card.id) {
            return Some(for_card(card, card_browser()));
        }
    }

    None
}

fn non_card_position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    id: GameObjectId,
) -> Option<ObjectPosition> {
    let current_prompt = prompts::current(game, builder.user_side);
    if let Some(GamePrompt::CardSelector(browser)) = current_prompt {
        let target = match browser.target {
            SelectorPromptTarget::DiscardPile => GameObjectId::DiscardPile(builder.user_side),
            SelectorPromptTarget::DeckTop => GameObjectId::Deck(builder.user_side),
            SelectorPromptTarget::DeckShuffled => GameObjectId::Deck(builder.user_side),
        };
        if id == target {
            return Some(for_sorting_key(
                0,
                Position::BrowserDragTarget(ObjectPositionBrowserDragTarget {}),
            ));
        }
    }

    raid_position_override(game, id)
}

fn raid_position_override(game: &GameState, id: GameObjectId) -> Option<ObjectPosition> {
    if let Some(raid_data) = &game.raid {
        match raid_display_state::build(game) {
            RaidDisplayState::None => None,
            RaidDisplayState::Defenders(defenders) => {
                browser_position(id, raid(), raid_browser(game, raid_data, defenders))
            }
            RaidDisplayState::Access => {
                browser_position(id, card_browser(), raid_access_browser(game, raid_data))
            }
        }
    } else {
        None
    }
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
            result.push(GameObjectId::Deck(Side::Covenant));
        }
        RoomId::Sanctum => {
            result.push(GameObjectId::Character(Side::Covenant));
        }
        RoomId::Crypt => {
            result.push(GameObjectId::DiscardPile(Side::Covenant));
        }
        _ => {}
    }

    result.extend(game.occupants(raid.target).map(|card| GameObjectId::CardId(card.id)));
    result.extend(defenders.iter().map(|card_id| GameObjectId::CardId(*card_id)));
    result.push(GameObjectId::Character(Side::Riftcaller));
    result
}

fn raid_access_browser(game: &GameState, raid: &RaidData) -> Vec<GameObjectId> {
    match raid.target {
        RoomId::Sanctum => {
            game.hand(Side::Covenant).map(|card| GameObjectId::CardId(card.id)).collect()
        }
        RoomId::Crypt => {
            game.discard_pile(Side::Covenant).map(|card| GameObjectId::CardId(card.id)).collect()
        }
        _ => raid.accessed.iter().map(|card_id| GameObjectId::CardId(*card_id)).collect(),
    }
}

fn character_facing_direction_for_side(
    builder: &ResponseBuilder,
    game: &GameState,
    side: Side,
) -> Option<GameCharacterFacingDirection> {
    if let Some(raid) = &game.raid {
        if matches!(raid_display_state::build(game), RaidDisplayState::Defenders(_))
            && builder.state.display_preference != Some(DisplayPreference::ShowArenaView(true))
        {
            if side == Side::Riftcaller {
                return Some(GameCharacterFacingDirection::Right);
            }

            if side == Side::Covenant && raid.target == RoomId::Sanctum {
                return Some(GameCharacterFacingDirection::Left);
            }
        }
    }

    Some(if builder.user_side == side {
        GameCharacterFacingDirection::Up
    } else {
        GameCharacterFacingDirection::Down
    })
}
