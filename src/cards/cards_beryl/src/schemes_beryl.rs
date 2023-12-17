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

use card_helpers::{abilities, costs, text, text_helpers, this};
use core_data::game_primitives::{CardType, GameObjectId, Rarity, School, Side};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_definition::{
    Ability, ActivatedAbility, CardConfigBuilder, CardDefinition, SchemePoints,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardPosition;
use game_data::special_effects::{SoundEffect, TimedEffect, TimedEffectData};
use game_data::state_machine_data::GiveCurseOptions;
use game_data::text::TextToken::*;
use rules::visual_effects::VisualEffects;
use rules::{curses, leylines, mana, mutations};

pub fn ethereal_form(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EtherealForm,
        sets: vec![CardSetName::Beryl],
        cost: costs::scheme(),
        image: assets::covenant_card(meta, "ethereal_form"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![ActivatedAbility::new(
            costs::actions(1),
            text![
                text!["Shuffle this card into the vault"],
                text![
                    "The Covenant may use this ability while this card is",
                    "in the Riftcaller's score area"
                ],
                meta.upgraded_only_text(text![GainMana(1)])
            ],
        )
        .delegate(this::can_activate(|g, s, _, flag| {
            flag.add_permission(
                g.card(s.card_id()).position() == CardPosition::Scored(Side::Riftcaller),
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

pub fn echoing_cacophony(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EchoingCacophony,
        sets: vec![CardSetName::Beryl],
        cost: costs::scheme(),
        image: assets::covenant_card(meta, "echoing_cacophony"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Beyond,
        rarity: Rarity::Uncommon,
        abilities: abilities::some(vec![
            Some(Ability::new_with_delegate(
                text_helpers::named_trigger(
                    Score,
                    text!["Give the Riftcaller 2 curses until", Dawn],
                ),
                this::on_scored_by_covenant(|g, s, _| {
                    curses::give_curses_with_options(
                        g,
                        s,
                        2,
                        GiveCurseOptions { for_turn: Some(g.info.turn) },
                    )
                }),
            )),
            meta.is_upgraded.then(|| {
                Ability::new_with_delegate(
                    text_helpers::named_trigger(Score, text![GainMana(2)]),
                    this::on_scored_by_covenant(|g, s, _| {
                        mana::gain(g, s.side(), 2);
                        Ok(())
                    }),
                )
            }),
        ]),
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 2, points: 10 })
            .build(),
    }
}

pub fn solidarity(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Solidarity,
        sets: vec![CardSetName::Beryl],
        cost: costs::scheme(),
        image: assets::covenant_card(meta, "solidarity"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Uncommon,
        abilities: vec![Ability::new_with_delegate(
            text_helpers::named_trigger(
                Score,
                text![text![GainMana(7)], text!["Give the Riftcaller a", Leyline]],
            ),
            this::on_scored_by_covenant(|g, s, _| {
                mana::gain(g, s.side(), 7);
                leylines::give(g, s.ability_id(), 1)
            }),
        )],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 2, points: 10 })
            .build(),
    }
}

pub fn brilliant_gambit(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::BrilliantGambit,
        sets: vec![CardSetName::Beryl],
        cost: costs::scheme(),
        image: assets::covenant_card(meta, "brilliant_gambit"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Uncommon,
        abilities: vec![
            Ability::new_with_delegate(
                text_helpers::named_trigger(
                    Score,
                    text![
                        "Remove",
                        meta.upgrade(text!["a", Leyline], text!["all", Leylines]),
                        "from the Riftcaller"
                    ],
                ),
                this::on_scored_by_covenant(|g, s, _| {
                    leylines::remove(g, s.ability_id(), s.upgrade(1, g.riftcaller.leylines))
                }),
            ),
            Ability::new_with_delegate(
                text!["When the Riftcaller scores this scheme, give them a", Leyline],
                this::on_scored_by_riftcaller(|g, s, _| {
                    VisualEffects::new()
                        .ability_alert(s)
                        .timed_effect(
                            GameObjectId::Character(Side::Riftcaller),
                            TimedEffectData::new(TimedEffect::MagicCircles1(6))
                                .scale(2.0)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagic_Buff01"))
                                .effect_color(design::YELLOW_900),
                        )
                        .apply(g);
                    leylines::give(g, s.ability_id(), 1)
                }),
            ),
        ],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 2, points: 10 })
            .build(),
    }
}

pub fn ritual_of_binding(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::RitualOfBinding,
        sets: vec![CardSetName::Beryl],
        cost: costs::scheme(),
        image: assets::covenant_card(meta, "ritual_of_binding"),
        card_type: CardType::Scheme,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text_helpers::named_trigger(Score, text![GainMana(meta.upgrade(5, 7))]),
                this::on_scored_by_covenant(|g, s, _| {
                    mana::gain(g, Side::Covenant, s.upgrade(5, 7));
                    Ok(())
                }),
            ),
            Ability::new(text![
                "The Riftcaller must",
                PayMana(meta.upgrade(5, 7)),
                "to score this card"
            ])
            .delegate(this::score_accessed_card_cost(|_, s, _, cost| {
                cost.add_mana_cost(s.upgrade(5, 7))
            })),
        ],
        config: CardConfigBuilder::new()
            .scheme_points(SchemePoints { progress_requirement: 5, points: 30 })
            .build(),
    }
}
