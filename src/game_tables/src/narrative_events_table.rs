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

use std::collections::HashMap;

use adventure_data::adventure_effect_data::{
    AdventureEffect, AdventureEffectData, AdventureEffectKind, DeckCardAction, DeckCardEffect,
};
use adventure_data::narrative_event_data::{NarrativeEventChoice, NarrativeEventData};
use anyhow::Result;
use core_data::adventure_primitives::{CardFilterId, Coins, NarrativeChoiceId, NarrativeEventId};
use core_data::game_primitives::Sprite;
use game_data::card_name::CardName;
use with_error::{fail, verify, WithError};

use crate::csv_datatypes::{NarrativeEventDetailsRow, NarrativeEventEntryKind, NarrativeEventRow};

pub fn build(
    rows: Vec<NarrativeEventRow>,
    details: Vec<NarrativeEventDetailsRow>,
) -> Result<HashMap<NarrativeEventId, NarrativeEventData>> {
    let mut result = HashMap::new();
    for row in rows.iter() {
        let id = NarrativeEventId::new(row.id);
        let details = find_details(&details, id);
        result.insert(
            id,
            NarrativeEventData {
                image: Sprite { address: row.image_path.clone() },
                description: find_entry(&details, NarrativeEventEntryKind::Introduction)?
                    .description,
                choices: all_choice_ids(&details)
                    .into_iter()
                    .map(|i| build_choice(&details, i))
                    .collect::<Result<HashMap<_, _>>>()?,
            },
        );
    }

    Ok(result)
}

fn find_details(
    details: &[NarrativeEventDetailsRow],
    id: NarrativeEventId,
) -> Vec<NarrativeEventDetailsRow> {
    details.iter().filter(|row| row.id == id.value).cloned().collect()
}

fn all_choice_ids(details: &[NarrativeEventDetailsRow]) -> Vec<u32> {
    let mut result = details.iter().filter_map(|row| row.choice_id).collect::<Vec<_>>();
    result.sort();
    result.dedup();
    result
}

fn build_choice(
    details: &[NarrativeEventDetailsRow],
    choice_id: u32,
) -> Result<(NarrativeChoiceId, NarrativeEventChoice)> {
    let id = NarrativeChoiceId::new(choice_id);
    Ok((
        id,
        NarrativeEventChoice {
            choice_description: find_choice_entry(details, NarrativeEventEntryKind::Choice, id)?
                .description,
            result_description: find_choice_entry(details, NarrativeEventEntryKind::Outcome, id)?
                .description,
            skill: find_choice_entry(details, NarrativeEventEntryKind::Choice, id)?.required_skill,
            costs: effect_list(details, choice_id, NarrativeEventEntryKind::Cost)?,
            rewards: effect_list(details, choice_id, NarrativeEventEntryKind::Reward)?,
        },
    ))
}

fn effect_list(
    details: &[NarrativeEventDetailsRow],
    choice_id: u32,
    kind: NarrativeEventEntryKind,
) -> Result<Vec<AdventureEffectData>> {
    details
        .iter()
        .filter_map(|row| {
            if row.choice_id != Some(choice_id) || row.entry_kind != kind {
                None
            } else {
                row.effect_kind.map(|effect| build_effect(row, effect))
            }
        })
        .collect::<Result<Vec<_>>>()
}

fn find_entry(
    details: &[NarrativeEventDetailsRow],
    kind: NarrativeEventEntryKind,
) -> Result<NarrativeEventDetailsRow> {
    let vec = details.iter().filter(|row| row.entry_kind == kind).collect::<Vec<_>>();
    verify!(vec.len() == 1, "Expected exactly one row of kind {kind:?} but got {}", vec.len());
    Ok(vec[0].clone())
}

fn find_choice_entry(
    details: &[NarrativeEventDetailsRow],
    kind: NarrativeEventEntryKind,
    id: NarrativeChoiceId,
) -> Result<NarrativeEventDetailsRow> {
    let vec = details
        .iter()
        .filter(|row| row.entry_kind == kind && row.choice_id == Some(id.value))
        .collect::<Vec<_>>();
    verify!(vec.len() == 1, "Expected exactly one row of kind {kind:?} but got {}", vec.len());
    Ok(vec[0].clone())
}

fn build_effect(
    row: &NarrativeEventDetailsRow,
    kind: AdventureEffectKind,
) -> Result<AdventureEffectData> {
    let effect = match kind {
        AdventureEffectKind::Draft => {
            AdventureEffect::Draft(resolve_card_filter(row.card_filter_id)?)
        }
        AdventureEffectKind::Shop => {
            AdventureEffect::Shop(resolve_card_filter(row.card_filter_id)?)
        }
        AdventureEffectKind::NarrativeEvent => {
            fail!("Not supported")
        }
        AdventureEffectKind::Battle => AdventureEffect::Battle,
        AdventureEffectKind::GainCoins => {
            AdventureEffect::GainCoins(Coins(resolve_quantity(row.quantity)?))
        }
        AdventureEffectKind::LoseCoins => {
            AdventureEffect::LoseCoins(Coins(resolve_quantity(row.quantity)?))
        }
        AdventureEffectKind::LoseAllCoins => AdventureEffect::LoseAllCoins,
        AdventureEffectKind::GainArcanite => {
            AdventureEffect::GainArcanite(resolve_quantity(row.quantity)?)
        }
        AdventureEffectKind::PickCardForEffect => AdventureEffect::PickCardForEffect(
            resolve_card_filter(row.card_filter_id)?,
            DeckCardEffect {
                action: resolve_deck_card_action(row.deck_card_action)?,
                cost: None,
                times: row.quantity.unwrap_or(1),
            },
        ),
        AdventureEffectKind::KnownRandomCardEffect => AdventureEffect::KnownRandomCardEffect(
            resolve_card_filter(row.card_filter_id)?,
            resolve_deck_card_action(row.deck_card_action)?,
        ),
        AdventureEffectKind::UnknownRandomCardEffect => AdventureEffect::UnknownRandomCardEffect(
            resolve_card_filter(row.card_filter_id)?,
            resolve_deck_card_action(row.deck_card_action)?,
        ),
        AdventureEffectKind::ApplyCardEffectToAllMatching => {
            AdventureEffect::ApplyCardEffectToAllMatching(
                resolve_card_filter(row.card_filter_id)?,
                resolve_deck_card_action(row.deck_card_action)?,
            )
        }
        AdventureEffectKind::AddMapTiles => {
            AdventureEffect::AddMapTiles(resolve_quantity(row.quantity)?)
        }
        AdventureEffectKind::AddDraftTiles => {
            AdventureEffect::AddDraftTiles(resolve_quantity(row.quantity)?)
        }
        AdventureEffectKind::AddNarrativeTiles => {
            AdventureEffect::AddNarrativeTiles(resolve_quantity(row.quantity)?)
        }
        AdventureEffectKind::GainKnownFixedCard => AdventureEffect::GainKnownFixedCard(
            resolve_card_name(row.card_name)?,
            row.quantity.unwrap_or(1),
        ),
        AdventureEffectKind::GainKnownRandomCard => AdventureEffect::GainKnownRandomCard(
            resolve_card_filter(row.card_filter_id)?,
            row.quantity.unwrap_or(1),
        ),
        AdventureEffectKind::GainUnknownRandomCard => AdventureEffect::GainUnknownRandomCard(
            resolve_card_filter(row.card_filter_id)?,
            row.quantity.unwrap_or(1),
        ),
        AdventureEffectKind::PickCardToLose => AdventureEffect::PickCardToLose(
            resolve_card_filter(row.card_filter_id)?,
            row.quantity.unwrap_or(1),
        ),
        AdventureEffectKind::LoseKnownRandomCard => AdventureEffect::LoseKnownRandomCard(
            resolve_card_filter(row.card_filter_id)?,
            row.quantity.unwrap_or(1),
        ),
        AdventureEffectKind::LoseUnknownRandomCard => AdventureEffect::LoseUnknownRandomCard(
            resolve_card_filter(row.card_filter_id)?,
            row.quantity.unwrap_or(1),
        ),
    };

    Ok(AdventureEffectData { effect, description: row.description.clone() })
}

fn resolve_quantity(quantity: Option<u32>) -> Result<u32> {
    quantity.with_error(|| "Expected quantity field")
}

fn resolve_deck_card_action(action: Option<DeckCardAction>) -> Result<DeckCardAction> {
    action.with_error(|| "Expected DeckCardAction")
}

fn resolve_card_filter(id: Option<u32>) -> Result<CardFilterId> {
    Ok(CardFilterId::new(id.with_error(|| "Expected CardFilterId")?))
}

fn resolve_card_name(name: Option<CardName>) -> Result<CardName> {
    name.with_error(|| "Expected CardName")
}
