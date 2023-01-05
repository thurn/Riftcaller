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

use std::collections::HashMap;

use anyhow::Result;
use data::card_name::CardName;
use data::deck::Deck;
use data::player_data::PlayerData;
use data::primitives::{DeckIndex, School, Side};
use data::user_actions::OldDeckEditorAction;
use with_error::{fail, WithError};

use crate::pick_deck_name;
use crate::pick_deck_name::DECK_NAME_INPUT;

pub fn handle(
    player: &mut PlayerData,
    action: OldDeckEditorAction,
    request_fields: &HashMap<String, String>,
) -> Result<()> {
    match action {
        OldDeckEditorAction::CreateDeck(side, school) => {
            let deck_name = match request_fields.get(DECK_NAME_INPUT) {
                Some(name) if !name.trim().is_empty() => name.clone(),
                _ => pick_deck_name::default_deck_name(side, school),
            };
            player.decks.push(Deck {
                index: DeckIndex::new(player.decks.len()),
                name: deck_name,
                owner_id: player.id,
                side,
                identity: default_identity(side, school)?,
                cards: HashMap::new(),
            });
        }
        OldDeckEditorAction::AddToDeck(card_name, deck_id) => {
            player.deck_mut(deck_id)?.cards.entry(card_name).and_modify(|e| *e += 1).or_insert(1);
        }
        OldDeckEditorAction::RemoveFromDeck(card_name, deck_id) => {
            let deck = player.deck_mut(deck_id)?;
            let count = *deck.cards.get(&card_name).with_error(|| "Card not present")?;
            match count {
                0 => fail!("Card count is zero"),
                1 => {
                    deck.cards.remove(&card_name);
                }
                _ => {
                    deck.cards.insert(card_name, count - 1);
                }
            }
        }
    }

    Ok(())
}

fn default_identity(side: Side, school: School) -> Result<CardName> {
    Ok(match (side, school) {
        (Side::Overlord, School::Law) => CardName::NoIdentityOverlordLaw,
        (Side::Overlord, School::Shadow) => CardName::NoIdentityOverlordShadow,
        (Side::Overlord, School::Primal) => CardName::NoIdentityOverlordPrimal,
        (Side::Champion, School::Law) => CardName::NoIdentityChampionLaw,
        (Side::Champion, School::Shadow) => CardName::NoIdentityChampionShadow,
        (Side::Champion, School::Primal) => CardName::NoIdentityChampionPrimal,
        _ => fail!("Neutral school not supported"),
    })
}
