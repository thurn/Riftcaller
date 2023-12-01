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

//! Helpers for converting between server & client data formats.

use anyhow::Result;
use core_data::adventure_primitives::TilePosition;
use core_data::game_primitives::{
    AbilityId, AbilityIndex, CardId, GameObjectId, Milliseconds, RoomId, Side, Sprite,
};
use game_data::character_preset::CharacterFacing;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::{
    CardIdentifier, GameCharacterFacingDirection, GameObjectIdentifier, MapPosition, PlayerSide,
    RoomIdentifier, SpriteAddress, TimeValue,
};
use with_error::fail;

use crate::response_builder::ResponseBuilder;

pub mod response_builder;

/// Possible custom cards which can be associated with a client card identifier
#[derive(Debug, Copy, Clone)]
pub enum CustomCardIdentifier {
    SummonProject = 1,
    Curse = 2,
    Dispel = 3,
    RoomSelector = 4,
    Wound = 5,
    Leyline = 6,
    StatusMarker = 7,
}

impl CustomCardIdentifier {
    pub fn from_u32(value: u32) -> Option<CustomCardIdentifier> {
        match value {
            1 => Some(CustomCardIdentifier::SummonProject),
            2 => Some(CustomCardIdentifier::Curse),
            3 => Some(CustomCardIdentifier::Dispel),
            4 => Some(CustomCardIdentifier::RoomSelector),
            _ => None,
        }
    }
}

pub fn card_identifier(card_id: CardId) -> CardIdentifier {
    // Maybe need to obfuscate this somehow?
    CardIdentifier {
        side: player_side(card_id.side),
        index: card_id.index as u32,
        ability_id: None,
        game_action: None,
    }
}

pub fn game_object_identifier(
    builder: &ResponseBuilder,
    identifier: impl Into<GameObjectId>,
) -> GameObjectIdentifier {
    GameObjectIdentifier {
        id: Some(match identifier.into() {
            GameObjectId::CardId(card_id) => Id::CardId(card_identifier(card_id)),
            GameObjectId::AbilityId(ability_id) => Id::CardId(ability_card_identifier(ability_id)),
            GameObjectId::Deck(side) => Id::Deck(builder.to_player_name(side)),
            GameObjectId::DiscardPile(side) => Id::DiscardPile(builder.to_player_name(side)),
            GameObjectId::Character(side) => Id::Character(builder.to_player_name(side)),
        }),
    }
}

pub fn ability_card_identifier(ability_id: AbilityId) -> CardIdentifier {
    CardIdentifier {
        ability_id: Some(ability_id.index.value() as u32),
        ..card_identifier(ability_id.card_id)
    }
}

/// Identifier for a card which provides the ability to summon a project in
/// play.
pub fn summon_project_card_identifier(card_id: CardId) -> CardIdentifier {
    CardIdentifier {
        game_action: Some(CustomCardIdentifier::SummonProject as u32),
        ..card_identifier(card_id)
    }
}

/// Identifier for a card representing an implicit game ability
pub fn custom_card_identifier(action: CustomCardIdentifier, number: u32) -> CardIdentifier {
    CardIdentifier {
        side: player_side(Side::Champion),
        index: number,
        game_action: Some(action as u32),
        ability_id: None,
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ServerCardId {
    /// Standard card
    CardId(CardId),
    /// Card representing an ability
    AbilityId(AbilityId),
    /// Card representing the implicit ability to summon a project
    SummonProject(CardId),
    /// Card representing the ability to remove a curse in hand
    CurseCard,
    /// Card representing the ability to destroy an evocation when the Champion
    /// is cursed
    DispelCard,
    /// Card representing selecting a room during a room selector prompt.
    RoomSelectorCard,
}

/// Converts a client [CardIdentifier] into a server [CardId] or [AbilityId].
pub fn server_card_id(card_id: CardIdentifier) -> Result<ServerCardId> {
    let result = CardId { side: side(card_id.side)?, index: card_id.index as usize };

    if let Some(action) = card_id.game_action.and_then(CustomCardIdentifier::from_u32) {
        return match action {
            CustomCardIdentifier::SummonProject => Ok(ServerCardId::SummonProject(result)),
            CustomCardIdentifier::Curse => Ok(ServerCardId::CurseCard),
            CustomCardIdentifier::Dispel => Ok(ServerCardId::DispelCard),
            CustomCardIdentifier::RoomSelector => Ok(ServerCardId::RoomSelectorCard),
            _ => fail!("Invalid CustomCardIdentifier"),
        };
    }

    card_id.ability_id.map_or(Ok(ServerCardId::CardId(result)), |index| {
        Ok(ServerCardId::AbilityId(AbilityId {
            card_id: result,
            index: AbilityIndex(index as usize),
        }))
    })
}

pub fn player_side(side: Side) -> i32 {
    match side {
        Side::Overlord => PlayerSide::Overlord as i32,
        Side::Champion => PlayerSide::Champion as i32,
    }
}

pub fn side(side: i32) -> Result<Side> {
    match PlayerSide::from_i32(side) {
        Some(PlayerSide::Overlord) => Ok(Side::Overlord),
        Some(PlayerSide::Champion) => Ok(Side::Champion),
        _ => fail!("Invalid player side"),
    }
}

pub fn room_identifier(room_id: RoomId) -> i32 {
    (match room_id {
        RoomId::Vault => RoomIdentifier::Vault,
        RoomId::Sanctum => RoomIdentifier::Sanctum,
        RoomId::Crypt => RoomIdentifier::Crypt,
        RoomId::RoomA => RoomIdentifier::RoomA,
        RoomId::RoomB => RoomIdentifier::RoomB,
        RoomId::RoomC => RoomIdentifier::RoomC,
        RoomId::RoomD => RoomIdentifier::RoomD,
        RoomId::RoomE => RoomIdentifier::RoomE,
    }) as i32
}

pub fn room_id(identifier: i32) -> Result<RoomId> {
    match RoomIdentifier::from_i32(identifier) {
        Some(RoomIdentifier::Vault) => Ok(RoomId::Vault),
        Some(RoomIdentifier::Sanctum) => Ok(RoomId::Sanctum),
        Some(RoomIdentifier::Crypt) => Ok(RoomId::Crypt),
        Some(RoomIdentifier::RoomA) => Ok(RoomId::RoomA),
        Some(RoomIdentifier::RoomB) => Ok(RoomId::RoomB),
        Some(RoomIdentifier::RoomC) => Ok(RoomId::RoomC),
        Some(RoomIdentifier::RoomD) => Ok(RoomId::RoomD),
        Some(RoomIdentifier::RoomE) => Ok(RoomId::RoomE),
        _ => fail!("Invalid RoomId: {:?}", identifier),
    }
}

/// Turns a [Sprite] into its protobuf equivalent
pub fn sprite(sprite: &Sprite) -> SpriteAddress {
    SpriteAddress { address: sprite.address.clone() }
}

pub fn milliseconds(milliseconds: u32) -> TimeValue {
    TimeValue { milliseconds }
}

pub fn map_position(p: TilePosition) -> MapPosition {
    MapPosition { x: p.x, y: p.y }
}

pub fn time_value(milliseconds: Milliseconds) -> TimeValue {
    TimeValue { milliseconds: milliseconds.0 }
}

pub fn game_character_facing_direction(facing: CharacterFacing) -> i32 {
    match facing {
        CharacterFacing::Up => GameCharacterFacingDirection::Up,
        CharacterFacing::Down => GameCharacterFacingDirection::Down,
        CharacterFacing::Left => GameCharacterFacingDirection::Left,
        CharacterFacing::Right => GameCharacterFacingDirection::Right,
    }
    .into()
}
