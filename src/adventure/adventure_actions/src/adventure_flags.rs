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

use adventure_data::adventure::{AdventureScreen, AdventureState};
use adventure_data::adventure_effect_data::DeckCardAction;
use adventure_data::narrative_event_data::{
    NarrativeChoiceState, NarrativeEventChoice, NarrativeEventStep,
};
use adventure_generator::card_filter;
use game_data::card_name::CardVariant;

pub fn can_apply_deck_card_effect(option: Option<&AdventureState>, card: CardVariant) -> bool {
    let Some(adventure) = option else { return false };
    if adventure.deck.all_cards().all(|v| v != card) {
        return false;
    }

    let Some(AdventureScreen::ApplyDeckEffect(filter, effect)) = adventure.screens.current() else {
        return false;
    };

    if effect.times == 0 {
        return false;
    }

    if !card_filter::matches(*filter, card) {
        return false;
    }

    match effect.action {
        DeckCardAction::DuplicateTo3Copies => {
            let Some(&count) = adventure.deck.cards.get(&card) else {
                return false;
            };

            count < 3
        }
        _ => true,
    }
}

pub fn can_end_narrative_event(state: &AdventureState) -> bool {
    let Some(AdventureScreen::NarrativeEvent(narrative_state)) = state.screens.current() else {
        return false;
    };
    let data = game_tables::narrative_event(narrative_state.id);

    match narrative_state.step {
        NarrativeEventStep::Introduction => true,
        NarrativeEventStep::ViewChoices => false,
        NarrativeEventStep::SelectChoice(choice_id) => {
            all_effects_applied(data.choice(choice_id), narrative_state.choice(choice_id))
        }
    }
}

/// Returns true if all of the costs and rewards for this [NarrativeEventChoice]
/// have either
/// 1) been applied or 2) are immediate and thus do not need to be applied.
pub fn all_effects_applied(data: &NarrativeEventChoice, state: &NarrativeChoiceState) -> bool {
    data.enumerate_costs()
        .chain(data.enumerate_rewards())
        .all(|(i, e)| state.effect(i).applied || e.effect.is_immediate())
}
