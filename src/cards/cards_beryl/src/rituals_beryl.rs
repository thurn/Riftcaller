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

use card_definition_data::ability_data::Ability;
use card_definition_data::card_definition::CardDefinition;
use card_definition_data::cards::CardDefinitionExt;
use card_helpers::card_selector_prompt_builder::CardSelectorPromptBuilder;
use card_helpers::play_card_browser_builder::PlayCardBrowserBuilder;
use card_helpers::{abilities, costs, delegates, requirements, show_prompt, text, this};
use core_data::game_primitives::{CardSubtype, CardType, HasCardId, Rarity, School, Side};
use game_data::card_configuration::{CardConfig, CardConfigBuilder};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardIdsExt;
use game_data::custom_card_state::CustomCardState;
use game_data::delegate_data::{CardInfoElementKind, CardStatusMarker, Scope};
use game_data::game_actions::ButtonPromptContext;
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::prompt_data::{
    CardSelectorPromptValidation, FromZone, PromptChoice, PromptContext, PromptData,
    SelectorPromptTarget,
};
use game_data::text::TextToken::*;
use rules::mutations::RealizeCards;
use rules::visual_effects::VisualEffects;
use rules::{curses, draw_cards, mutations, prompts};

pub fn equivalent_exchange(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EquivalentExchange,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::covenant_card(meta, "equivalent_exchange"),
        card_type: CardType::Ritual,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: abilities::some(vec![
            abilities::when_not_upgraded(meta, abilities::play_only_if_riftcaller_cursed()),
            Some(abilities::silent_can_play(|g, _, _, current| {
                current.add_constraint(
                    g.score_area(Side::Covenant).filter(|c| c.definition().is_scheme()).count() > 0
                        && g.score_area(Side::Riftcaller)
                            .filter(|c| c.definition().is_scheme())
                            .count()
                            > 0,
                )
            })),
            Some(
                Ability::new_with_delegate(
                    text![
                        "Swap a scheme in your score area with one in the Riftcaller's score area"
                    ],
                    this::on_played(|g, s, _| {
                        // Note that second option is shown first on prompt stack
                        prompts::push_with_data(g, Side::Covenant, s, PromptData::Index(1));
                        prompts::push_with_data(g, Side::Covenant, s, PromptData::Index(0));
                        Ok(())
                    }),
                )
                .delegate(this::prompt(|g, s, source, _| {
                    let PromptData::Index(i) = source.data else {
                        return None;
                    };

                    if i == 0 {
                        show_prompt::with_context_and_choices(
                            ButtonPromptContext::CardToGiveToOpponent,
                            g.score_area(Side::Covenant)
                                .filter(|c| c.definition().is_scheme())
                                .map(|c| {
                                    PromptChoice::new()
                                        .effect(GameEffect::SelectCardForPrompt(s.side(), c.id))
                                        .anchor_card(c.id)
                                })
                                .collect(),
                        )
                    } else if i == 1 {
                        show_prompt::with_context_and_choices(
                            ButtonPromptContext::CardToTakeFromOpponent,
                            g.score_area(Side::Riftcaller)
                                .filter(|c| c.definition().is_scheme())
                                .map(|c| {
                                    PromptChoice::new()
                                        .effect(GameEffect::SwapWithSelected(s.side(), c.id))
                                        .anchor_card(c.id)
                                })
                                .collect(),
                        )
                    } else {
                        None
                    }
                })),
            ),
        ]),
        config: CardConfig::default(),
    }
}

pub fn lightbond(meta: CardMetadata) -> CardDefinition {
    fn targeted(game: &GameState, scope: Scope, id: &impl HasCardId) -> bool {
        let accessed = id.card_id();
        game.card(scope)
            .custom_state
            .targets_contain(game.card(accessed).last_card_play_id, accessed)
    }

    CardDefinition {
        name: CardName::Lightbond,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::covenant_card(meta, "lightbond"),
        card_type: CardType::Ritual,
        subtypes: vec![CardSubtype::Fabrication],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Rare,
        abilities: abilities::some(vec![
            Some(
                Ability::new(text![
                    text!["Play a scheme face-up"],
                    text!["When the Riftcaller accesses this scheme, give them", 2, Curses],
                ])
                .delegate(this::on_played(|g, s, _| {
                    prompts::push(g, Side::Covenant, s);
                    Ok(())
                }))
                .delegate(this::prompt(|g, s, _, _| {
                    PlayCardBrowserBuilder::new(
                        s,
                        FromZone::Hand,
                        g.hand(s.side()).filter(|c| c.definition().is_scheme()).card_ids(),
                    )
                    .build()
                }))
                .delegate(delegates::on_played(
                    requirements::matching_play_browser,
                    |g, s, played| {
                        mutations::turn_face_up(g, played.card_id);
                        g.card_mut(s).custom_state.push(CustomCardState::TargetCard {
                            target_card: played.card_id,
                            play_id: played.card_play_id,
                        });
                        Ok(())
                    },
                ))
                .delegate(delegates::on_card_access(targeted, |g, s, _| {
                    VisualEffects::new().ability_alert(s).apply(g);
                    curses::give_curses(g, s.ability_id(), 2)
                }))
                .delegate(delegates::on_query_card_status_markers(
                    targeted,
                    |_, s, _, mut markers| {
                        markers.push(CardStatusMarker {
                            source: s.ability_id(),
                            marker_kind: CardInfoElementKind::NegativeEffect,
                            text: text![
                                "When the Riftcaller accesses this scheme, give them",
                                2,
                                Curses
                            ],
                        });
                        markers
                    },
                )),
            ),
            Some(abilities::silent_can_play(|g, s, _, current| {
                current.add_constraint(g.hand(s.side()).any(|card| card.definition().is_scheme()))
            })),
            meta.is_upgraded.then(|| abilities::gain_mana_on_play::<2>()),
        ]),
        config: CardConfig::default(),
    }
}

pub fn foresee(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Foresee,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::covenant_card(meta, "foresee"),
        card_type: CardType::Ritual,
        subtypes: vec![CardSubtype::Augury],
        side: Side::Covenant,
        school: School::Beyond,
        rarity: Rarity::Uncommon,
        abilities: vec![Ability::new(text![
            text!["Look at the top", 5, "cards of the", Vault, "and arrange them in any order"],
            meta.upgraded_only_text(text!["Draw a card"])
        ])
        .delegate(this::on_played(|g, s, _| {
            let cards = mutations::realize_top_of_deck(
                g,
                Side::Covenant,
                5,
                RealizeCards::SetVisibleToOwner,
            )?;
            prompts::push_with_data(g, Side::Covenant, s, PromptData::Cards(cards));
            Ok(())
        }))
        .delegate(this::prompt(|_, s, source, _| {
            let PromptData::Cards(cards) = &source.data else {
                return None;
            };
            CardSelectorPromptBuilder::new(s, SelectorPromptTarget::DeckTop)
                .subjects(cards.clone())
                .context(PromptContext::ReorderTopOfVault)
                .can_reorder(true)
                .validation(CardSelectorPromptValidation::AllSubjects)
                .build()
        }))
        .delegate(this::on_card_selector_submitted(|g, s, _| {
            if s.is_upgraded() {
                draw_cards::run(g, Side::Covenant, 1, s.initiated_by())?;
            }
            Ok(())
        }))],
        config: CardConfig::default(),
    }
}

pub fn dusks_ascension(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::DusksAscension,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::covenant_card(meta, "dusks_ascension"),
        card_type: CardType::Ritual,
        subtypes: vec![CardSubtype::Fabrication],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Uncommon,
        abilities: vec![Ability::new(text![
            text![
                "Place",
                meta.upgrade(3, 4),
                "progress counters on the card occupying target room"
            ],
            text!["Turn that card face up"],
            text!["You cannot score it until your next turn"]
        ])
        .delegate(this::on_played(|g, s, played| {
            let turn = g.info.turn;
            let room_id = played.target.room_id()?;
            for card_id in g.occupants(room_id).card_ids() {
                mutations::turn_face_up(g, card_id);
                g.card_mut(s)
                    .custom_state
                    .push(CustomCardState::TargetCardForTurn { target_card: card_id, turn });
            }
            mutations::progress_card_occupying_room(g, room_id, s.initiated_by(), s.upgrade(3, 4))?;
            Ok(())
        }))
        .delegate(delegates::can_covenant_score_scheme(
            requirements::card_targeted_for_this_turn_cycle,
            delegates::disallow_ability,
        ))
        .delegate(this::at_dusk(|g, _, _| mutations::check_for_covenant_scoring_schemes(g)))
        .delegate(delegates::status_markers(
            requirements::card_targeted_for_this_turn_cycle,
            |_, s, _, mut markers| {
                markers.push(CardStatusMarker {
                    source: s.ability_id(),
                    marker_kind: CardInfoElementKind::NegativeEffect,
                    text: text!["Cannot be scored"],
                });
                markers
            },
        ))],
        config: CardConfigBuilder::new().custom_targeting(requirements::occupied_room()).build(),
    }
}

pub fn foretell_fate(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ForetellFate,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::covenant_card(meta, "foretell_fate"),
        card_type: CardType::Ritual,
        subtypes: vec![CardSubtype::Incantation],
        side: Side::Covenant,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![Ability::new(text![
            text!["Shuffle any number of cards from the", Sanctum, "into the", Vault],
            text!["Draw that many cards", meta.upgrade("", "plus one")]
        ])
        .delegate(this::on_played(|g, s, _| {
            prompts::push(g, Side::Covenant, s);
            Ok(())
        }))
        .delegate(this::prompt(|g, s, _, _| {
            CardSelectorPromptBuilder::new(s, SelectorPromptTarget::DeckShuffled)
                .subjects(g.hand(Side::Covenant).card_ids())
                .context(PromptContext::ShuffleIntoVault)
                .build()
        }))
        .delegate(this::on_card_selector_submitted(|g, s, event| {
            draw_cards::run(
                g,
                Side::Covenant,
                event.subjects.len() as u32 + s.upgrade(0, 1),
                s.initiated_by(),
            )
        }))],
        config: CardConfig::default(),
    }
}
