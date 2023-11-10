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

use card_helpers::{costs, text, this};
use game_data::card_definition::{
    ActivatedAbility, CardConfigBuilder, CardDefinition, SchemePoints,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardPosition;
use game_data::primitives::{CardType, Rarity, School, Side};
use game_data::text::TextToken::*;
use rules::{mana, mutations};

pub fn ethereal_form(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EtherealForm,
        sets: vec![CardSetName::Beryl],
        cost: costs::scheme(),
        image: assets::overlord_card(meta, "ethereal_form"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![ActivatedAbility::new(
            costs::actions(1),
            text![
                text!["Shuffle this card into the vault"],
                text![
                    "The Overlord may use this ability while this card is",
                    "in the Champion's score area"
                ],
                meta.upgraded_only_text(text![GainMana(1)])
            ],
        )
        .delegate(this::can_activate(|g, s, _, flag| {
            flag.add_permission(
                g.card(s.card_id()).position() == CardPosition::Scored(Side::Champion),
            )
        }))
        .delegate(this::on_activated(|g, s, _| {
            mutations::shuffle_into_deck(g, s.side(), &[s.card_id()])?;
            if s.is_upgraded() {
                mana::gain(g, s.side(), 1);
            }
            Ok(())
        }))
        .build()],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 2, points: 10 })
            .build(),
    }
}
