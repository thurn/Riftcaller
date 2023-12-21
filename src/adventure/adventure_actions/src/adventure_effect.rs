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

use adventure_data::adventure::{AdventureScreen, AdventureState};
use adventure_data::adventure_effect_data::AdventureEffect;
use adventure_data::narrative_event_data::{NarrativeEventState, NarrativeEventStep};
use adventure_generator::{battle_generator, card_filter};
use anyhow::Result;
use game_data::card_name::CardVariant;
use with_error::WithError;

pub fn apply(
    state: &mut AdventureState,
    effect: AdventureEffect,
    known_card: Option<CardVariant>,
) -> Result<()> {
    match effect {
        AdventureEffect::Draft(selector) => {
            let data = card_filter::draft_choices(state, selector);
            state.screens.push(AdventureScreen::Draft(data));
        }
        AdventureEffect::Shop(selector) => {
            let data = card_filter::shop_choices(state, selector);
            state.screens.push(AdventureScreen::Shop(data));
        }
        AdventureEffect::NarrativeEvent(id) => {
            state.screens.push(AdventureScreen::NarrativeEvent(NarrativeEventState {
                id,
                step: NarrativeEventStep::Introduction,
                choices: HashMap::new(),
            }));
        }
        AdventureEffect::Battle => state
            .screens
            .push(AdventureScreen::Battle(battle_generator::create(state.side.opponent()))),
        AdventureEffect::PickCardForEffect(filter, effect) => {
            state.screens.push(AdventureScreen::ApplyDeckEffect(filter, effect))
        }
        AdventureEffect::LoseKnownRandomCard(_, copies) => {
            state
                .deck
                .cards
                .entry(known_card.with_error(|| "Expected known_card")?)
                .and_modify(|count| *count = count.saturating_sub(copies));
        }
        _ => {
            panic!("Not implemented {effect:?}")
        }
    }
    Ok(())
}
