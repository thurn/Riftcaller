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

use adventure_data::adventure::{AdventureConfiguration, AdventureScreen, AdventureState};
use adventure_data::adventure_action::NarrativeEffectIndex;
use adventure_data::adventure_effect_data::{AdventureEffect, AdventureEffectData};
use adventure_data::narrative_event_data::{
    NarrativeEffectState, NarrativeEventData, NarrativeEventState, NarrativeEventStep,
};
use adventure_generator::card_filter;
use anyhow::Result;
use core_data::adventure_primitives::NarrativeChoiceId;
use game_data::deck::Deck;
use with_error::{fail, verify};

use crate::{adventure_effect, adventure_flags};

/// Handles a request from a user to advance to a given step within a narrative
/// event screen.
pub fn set_narrative_step(state: &mut AdventureState, step: NarrativeEventStep) -> Result<()> {
    let Some(AdventureScreen::NarrativeEvent(narrative)) = state.screens.current_mut() else {
        fail!("Expected active narrative event screen");
    };
    let data = game_tables::narrative_event(narrative.id);

    match step {
        NarrativeEventStep::Introduction => narrative.step = NarrativeEventStep::Introduction,
        NarrativeEventStep::ViewChoices => {
            reify_known_choices(&mut state.config, data, narrative, &state.deck);
            narrative.step = NarrativeEventStep::ViewChoices
        }
        NarrativeEventStep::SelectChoice(index) => {
            verify!(is_legal_choice(data, index), "Invalid choice!");
            narrative.choice_mut(index).selected = true;
            narrative.step = step;

            let choice = data.choice(index);
            let immediate_effects = choice
                .enumerate_costs()
                .chain(choice.enumerate_rewards())
                .filter_map(|(i, e)| e.effect.is_immediate().then_some(i))
                .collect::<Vec<_>>();

            for effect_index in immediate_effects {
                apply_narrative_effect(state, index, effect_index)?;
            }
        }
    }

    Ok(())
}

pub fn apply_narrative_effect(
    state: &mut AdventureState,
    choice_index: NarrativeChoiceId,
    effect_index: NarrativeEffectIndex,
) -> Result<()> {
    let Some(AdventureScreen::NarrativeEvent(narrative)) = state.screens.current_mut() else {
        fail!("Expected active narrative event screen");
    };

    let choice_state = narrative.choice_mut(choice_index).effect_mut(effect_index);
    choice_state.applied = true;
    let card = choice_state.known_card;
    let data = game_tables::narrative_event(narrative.id);
    let effect_data = data.choice(choice_index).effect(effect_index);
    let effect = effect_data.effect;

    adventure_effect::apply(state, effect, card)
}

pub fn end_narrative_event(state: &mut AdventureState) -> Result<()> {
    verify!(
        adventure_flags::can_end_narrative_event(state),
        "Cannot currently end narrative event"
    );
    state.screens.pop();
    Ok(())
}

/// Returns true if the player is allowed to pick the `index` option within the
/// provided [NarrativeEventData].
pub fn is_legal_choice(data: &NarrativeEventData, id: NarrativeChoiceId) -> bool {
    data.choices.contains_key(&id)
}

/// "Known random" choices are ones that are random each time this narrative
/// event is seen, but are known to the player before being selected.
///
/// This function picks values for known random choices when the user requests
/// to view the available choices for a narrative event.
fn reify_known_choices(
    config: &mut AdventureConfiguration,
    data: &NarrativeEventData,
    state: &mut NarrativeEventState,
    deck: &Deck,
) {
    for (id, choice) in &data.choices {
        for (index, cost) in choice.enumerate_costs() {
            reify_known_effect(config, cost, state.choice_mut(*id).effect_mut(index), deck);
        }
        for (index, effect) in choice.enumerate_rewards() {
            reify_known_effect(config, effect, state.choice_mut(*id).effect_mut(index), deck);
        }
    }
}

fn reify_known_effect(
    config: &mut AdventureConfiguration,
    effect_data: &AdventureEffectData,
    effect_state: &mut NarrativeEffectState,
    deck: &Deck,
) {
    match &effect_data.effect {
        AdventureEffect::LoseKnownRandomCard(selector, _) => {
            let choice = config.choose(card_filter::deck(deck, *selector));
            effect_state.known_card = choice;
        }
        _ => {}
    }
}
