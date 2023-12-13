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
    CardSelector, NarrativeEventChoice, NarrativeEventData, NarrativeEventStep,
};
use adventure_data::adventure_cost::AdventureCost;
use adventure_data::adventure_effect::{AdventureEffect, DeckCardEffect};
use core_data::adventure_primitives::Skill;
use core_data::game_primitives::{CardSubtype, CardType, Rarity};

pub fn generate() -> NarrativeEventData {
    NarrativeEventData {
        step: NarrativeEventStep::Introduction,
        description: "As the dust and debris swirl around the jagged peaks, \
        you find yourself face-to-face with the legendary Stormfeather Eagle, its eyes \
        ablaze with a fierce intelligence.\n\nThe air crackles with the power of this mythical \
        beast, and it's clear that only one of you will leave these heights as victor."
            .to_string(),
        choices: vec![
            NarrativeEventChoice {
                choice_description:
                    "\"With my sword drawn, I challenge the mighty eagle to a duel of strength!\""
                        .to_string(),
                result_description:
                    "The clash of your sword against the eagle's talons sends sparks \
            flying. After a mighty struggle, the eagle yields, bestowing upon you the ancient \
            Spell of the Mountain's Might, a secret kept by the high winds for eons."
                        .to_string(),
                skill: Some(Skill::Brawn),
                costs: vec![],
                effects: vec![AdventureEffect::Draft(
                    CardSelector::new().rarity(Rarity::Rare).card_type(CardType::Spell),
                )],
            },
            NarrativeEventChoice {
                choice_description: "\"I'll use the rocks for cover and move silently to find an \
                    advantage point over the creature.\""
                    .to_string(),
                result_description: "Slipping from boulder to boulder, your silent steps go \
                unnoticed. The eagle, confused, eventually perches atop a craggy spire, granting \
                you the chance to snatch a feather. This feather pulses with a magical blessing, \
                duplicating items in your possession."
                    .to_string(),
                skill: Some(Skill::Stealth),
                costs: vec![],
                effects: vec![
                    AdventureEffect::PickCardForEffect(DeckCardEffect::Duplicate(3)),
                    AdventureEffect::PickCardForEffect(DeckCardEffect::Duplicate(3)),
                    AdventureEffect::PickCardForEffect(DeckCardEffect::Duplicate(3)),
                ],
            },
            NarrativeEventChoice {
                choice_description: "\"I offer my blade to you, oh great Stormfeather, as a \
                token of respect and in exchange for safe passage.\""
                    .to_string(),
                result_description: "Placing your weapon upon an altar of stone, you step back. \
                The eagle swoops down, taking the offering in its beak before soaring away, \
                leaving behind a clear vision of a hidden shop on your map, tucked away in the \
                mountains, a place of rare and powerful artifacts."
                    .to_string(),
                skill: None,
                costs: vec![AdventureCost::LoseKnownRandomCard(
                    CardSelector::new().non_basic(true).card_subtype(CardSubtype::Weapon),
                )],
                effects: vec![AdventureEffect::Shop(CardSelector::new())],
            },
        ],
        selected_choices: vec![],
    }
}
