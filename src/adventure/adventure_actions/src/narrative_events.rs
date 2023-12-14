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

use adventure_data::adventure::{
    AdventureConfiguration, AdventureState, NarrativeEventData, NarrativeEventStep, TileEntity,
};
use adventure_data::adventure_action::NarrativeEffectIndex;
use adventure_data::adventure_effect::{AdventureEffect, AdventureEffectData};
use anyhow::Result;
use core_data::adventure_primitives::NarrativeChoiceIndex;
use game_data::deck::Deck;
use with_error::{fail, verify};

use crate::{adventure_effect, card_selector};

/// Handles a request from a user to advance to a given step within a narrative
/// event screen.
pub fn set_narrative_step(state: &mut AdventureState, step: NarrativeEventStep) -> Result<()> {
    let TileEntity::NarrativeEvent(data) = state.world_map.visiting_tile_mut()? else {
        fail!("Expected active narrative event screen");
    };

    match step {
        NarrativeEventStep::Introduction => data.step = NarrativeEventStep::Introduction,
        NarrativeEventStep::ViewChoices => {
            reify_known_choices(&mut state.config, data, &state.deck);
            data.step = NarrativeEventStep::ViewChoices
        }
        NarrativeEventStep::SelectChoice(index) => {
            verify!(is_legal_choice(data, index), "Invalid choice!");
            data.selected_choices.push(index);
            data.step = step;
        }
    }

    Ok(())
}

pub fn apply_narrative_effect(
    state: &mut AdventureState,
    choice_index: NarrativeChoiceIndex,
    effect_index: NarrativeEffectIndex,
) -> Result<()> {
    let TileEntity::NarrativeEvent(data) = state.world_map.visiting_tile_mut()? else {
        fail!("Expected active narrative event screen");
    };
    let choice = data.choice_mut(choice_index);
    choice.applied.push(effect_index);

    let effect_data = choice.effect(effect_index);
    let effect = effect_data.effect.clone();
    let card = effect_data.known_card;

    adventure_effect::apply(state, effect, card)
}

/// Returns true if the player is allowed to pick the `index` option within the
/// provided [NarrativeEventData].
pub fn is_legal_choice(data: &NarrativeEventData, index: NarrativeChoiceIndex) -> bool {
    index.value < data.choices.len()
}

/// "Known random" choices are ones that are random each time this narrative
/// event is seen, but are known to the player before being selected.
///
/// This function picks values for known random choices when the user requests
/// to view the available choices for a narrative event.
fn reify_known_choices(
    config: &mut AdventureConfiguration,
    data: &mut NarrativeEventData,
    deck: &Deck,
) {
    for choice in &mut data.choices {
        for cost in &mut choice.costs {
            reify_known_effect(config, cost, deck);
        }
        for effect in &mut choice.rewards {
            reify_known_effect(config, effect, deck);
        }
    }
}

fn reify_known_effect(
    config: &mut AdventureConfiguration,
    effect_data: &mut AdventureEffectData,
    deck: &Deck,
) {
    match &effect_data.effect {
        AdventureEffect::LoseKnownRandomCard(selector) => {
            let choice = config.choose(
                deck.cards.keys().filter(|&&variant| card_selector::matches(selector, variant)),
            );
            effect_data.known_card = choice.copied();
        }
        _ => {}
    }
}
