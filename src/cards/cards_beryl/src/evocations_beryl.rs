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

use std::iter;

use card_definition_data::ability_data::{Ability, ActivatedAbility};
use card_definition_data::card_definition::CardDefinition;
use card_definition_data::cards::CardDefinitionExt;
use card_helpers::play_card_browser_builder::PlayCardBrowserBuilder;
use card_helpers::{
    abilities, costs, delegates, history, in_play, raids, requirements, show_prompt, text,
    text_helpers, this,
};
use core_data::game_primitives::{
    CardId, CardSubtype, CardType, GameObjectId, InitiatedBy, Rarity, RoomId, School, Side,
};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::animation_tracker::GameAnimation;
use game_data::card_configuration::{CardConfig, CardConfigBuilder, TargetRequirement};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::{BanishedByCard, CardCounter, CardPosition};
use game_data::delegate_data::{CardInfoElementKind, CardStatusMarker};
use game_data::game_actions::{ButtonPromptContext, CardTarget};
use game_data::game_effect::GameEffect;
use game_data::history_data::HistoryEvent;
use game_data::prompt_data::{
    FromZone, PromptChoice, PromptChoiceLabel, PromptData, UnplayedAction,
};
use game_data::special_effects::{SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextElement;
use game_data::text::TextToken::*;
use raid_state::{custom_access, InitiateRaidOptions};
use rules::mutations::{OnZeroStored, RealizeCards};
use rules::visual_effects::{ShowAlert, VisualEffects};
use rules::{curses, destroy, draw_cards, flags, mana, mutations, prompts, visual_effects};

pub fn empyreal_chorus(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EmpyrealChorus,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::riftcaller_card(meta, "empyreal_chorus"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![ActivatedAbility::new(
            costs::sacrifice_and_action(),
            text![
                text!["Raid target outer room"],
                text![
                    "If successful,",
                    GainMana(meta.upgrade(8, 10)),
                    "instead of accessing cards"
                ]
            ],
        )
        .target_requirement(TargetRequirement::TargetRoom(|g, _, r| {
            r.is_outer_room() && flags::is_valid_raid_target(g, r)
        }))
        .delegate(this::on_activated(|g, s, activated| {
            raids::initiate_with_options(
                g,
                s,
                activated.target,
                InitiateRaidOptions { is_card_access_prevented: true },
            )
        }))
        .delegate(delegates::on_raid_successful(requirements::matching_raid, |g, s, _| {
            VisualEffects::new()
                .ability_alert(s)
                .timed_effect(
                    GameObjectId::Character(Side::Riftcaller),
                    TimedEffectData::new(TimedEffect::MagicCircles1(10))
                        .scale(4.0)
                        .sound(SoundEffect::LightMagic("RPG3_LightMagic_Cast02"))
                        .effect_color(design::YELLOW_900),
                )
                .apply(g);
            mana::gain(g, s.side(), s.upgrade(8, 10));
            Ok(())
        }))
        .build()],
        config: CardConfig::default(),
    }
}

pub fn starfield_omen(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::StarfieldOmen,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(4),
        image: assets::riftcaller_card(meta, "starfield_omen"),
        card_type: CardType::Evocation,
        subtypes: vec![CardSubtype::Mystic],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text!["When you sacrifice an artifact, draw a card"],
            in_play::on_card_sacrificed(|g, s, card_id| {
                if g.card(*card_id).definition().card_type == CardType::Artifact {
                    VisualEffects::new()
                        .timed_effect(
                            GameObjectId::CardId(s.card_id()),
                            TimedEffectData::new(TimedEffect::MagicCircles2(12))
                                .scale(1.5)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_Transform01"))
                                .effect_color(design::YELLOW_900),
                        )
                        .apply(g);

                    draw_cards::run(g, s.side(), 1, s.initiated_by())?;
                }
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn visitation(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Visitation,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(3),
        image: assets::riftcaller_card(meta, "visitation"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            // This is templated as an activated ability for clarity even though it's
            // secretly not.
            text![TextElement::Activated {
                cost: text![SacrificeCost],
                effect: text!["Prevent up to", meta.upgrade(2, 5), Damage]
            }],
            in_play::on_will_deal_damage(|g, s, damage| {
                if damage.source.side() == Side::Covenant {
                    prompts::push(g, Side::Riftcaller, s);
                }
                Ok(())
            }),
        )
        .delegate(this::prompt(|_, s, _, _| {
            show_prompt::with_context_and_choices(
                ButtonPromptContext::SacrificeToPreventDamage(s.card_id(), s.upgrade(2, 5)),
                vec![
                    PromptChoice::new()
                        .effect(GameEffect::SacrificeCard(s.card_id()))
                        .effect(GameEffect::PreventDamage(s.upgrade(2, 5))),
                    PromptChoice::new().effect(GameEffect::Continue),
                ],
            )
        }))],
        config: CardConfig::default(),
    }
}

pub fn backup_plan(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::BackupPlan,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::riftcaller_card(meta, "backup_plan"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![ActivatedAbility::new(
            costs::sacrifice(),
            text![
                text![Evade, "a minion"],
                meta.upgrade(text!["Lose all", ActionSymbol], text!["Lose", Actions(1)])
            ],
        )
        .delegate(this::can_activate(|g, _, _, flag| {
            flag.add_constraint(raids::active_encounter_prompt(g).is_some())
                .add_constraint(flags::can_evade_current_minion(g))
        }))
        .delegate(this::on_activated(|g, s, _| {
            mutations::evade_current_minion(g)?;
            mutations::lose_action_points_if_able(
                g,
                Side::Riftcaller,
                s.upgrade(g.riftcaller.actions, 1),
            )
        }))
        .build()],
        config: CardConfig::default(),
    }
}

pub fn planar_sanctuary(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::PlanarSanctuary,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(2),
        image: assets::riftcaller_card(meta, "planar_sanctuary"),
        card_type: CardType::Evocation,
        subtypes: vec![CardSubtype::Mystic, CardSubtype::Charge],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![
            Ability::new_with_delegate(
                text!["When a scheme is scored,", AddPowerCharges(meta.upgrade(2, 3))],
                in_play::on_card_scored(|g, s, _| {
                    VisualEffects::new()
                        .timed_effect(
                            GameObjectId::CardId(s.card_id()),
                            TimedEffectData::new(TimedEffect::MagicCircles1(3))
                                .scale(2.0)
                                .effect_color(design::BLUE_500),
                        )
                        .apply(g);

                    mutations::add_power_charges(g, s.card_id(), s.upgrade(2, 3))
                }),
            ),
            ActivatedAbility::new(
                costs::power_charges::<1>(),
                text![text!["Remove a curse"], text!["Draw a card"]],
            )
            .delegate(this::on_activated(|g, s, _| {
                curses::remove_curses(g, 1)?;
                draw_cards::run(g, s.side(), 1, s.initiated_by())?;
                Ok(())
            }))
            .build(),
            Ability::new(text!["You may activate abilities after being cursed or damaged"])
                .delegate(in_play::on_curse(|g, s, _| {
                    if g.card(s.card_id()).counters(CardCounter::PowerCharges) > 0 {
                        prompts::push(g, Side::Riftcaller, s);
                    }
                    Ok(())
                }))
                .delegate(in_play::on_damage(|g, s, _| {
                    if g.card(s.card_id()).counters(CardCounter::PowerCharges) > 0 {
                        prompts::push(g, Side::Riftcaller, s);
                    }
                    Ok(())
                }))
                .delegate(this::prompt(|_, _, _, _| show_prompt::priority_window())),
        ],
        config: CardConfig::default(),
    }
}

pub fn knowledge_of_the_beyond(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::KnowledgeOfTheBeyond,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::riftcaller_card(meta, "knowledge_of_the_beyond"),
        card_type: CardType::Evocation,
        subtypes: vec![CardSubtype::Augury],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Uncommon,
        abilities: vec![
            Ability::new(text_helpers::named_trigger(
                Play,
                text![Banish, "the top", 3, "cards of your deck"],
            ))
            .delegate(this::on_played(|g, s, play_card| {
                let cards = mutations::realize_top_of_deck(
                    g,
                    s.side(),
                    3,
                    RealizeCards::SetVisibleToOwner,
                )?;
                g.add_animation(|| GameAnimation::DrawCards(s.side(), cards.clone()));
                g.card_mut(s).custom_state.record_targets(play_card.card_play_id, &cards);
                mutations::move_cards(
                    g,
                    &cards,
                    CardPosition::Banished(Some(BanishedByCard {
                        source: s.card_id(),
                        play_id: play_card.card_play_id,
                    })),
                )
            })),
            ActivatedAbility::new(
                costs::sacrifice(),
                text![
                    text![
                        "Play a permanent from among those cards, reducing its cost by",
                        Mana(meta.upgrade(1, 4))
                    ],
                    text!["Discard the rest"]
                ],
            )
            .delegate(this::on_activated(|g, s, _| {
                let Some(play_id) = g.card(s).last_card_play_id else { return Ok(()) };
                let (permanents, spells): (Vec<CardId>, Vec<CardId>) = g
                    .card(s)
                    .custom_state
                    .targets(play_id)
                    .partition(|id| g.card(*id).definition().is_permanent());

                mutations::move_cards(g, &spells, CardPosition::DiscardPile(s.side()))?;
                prompts::push_with_data(g, Side::Riftcaller, s, PromptData::Cards(permanents));
                Ok(())
            }))
            .delegate(this::prompt(|_, s, source, _| {
                let PromptData::Cards(permanents) = &source.data else {
                    return None;
                };
                PlayCardBrowserBuilder::new(s, FromZone::Banished, permanents.clone())
                    .unplayed_action(UnplayedAction::Discard)
                    .build()
            }))
            .delegate(delegates::mana_cost(requirements::matching_play_browser, |_, s, _, cost| {
                cost.map(|c| c.saturating_sub(s.upgrade(1, 4)))
            }))
            .build(),
        ],
        config: CardConfig::default(),
    }
}

pub fn splinter_of_twilight(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SplinterOfTwilight,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(7, 4)),
        image: assets::riftcaller_card(meta, "splinter_of_twilight"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![
            Ability::new_with_delegate(
                text![
                    "When you successfully raid the",
                    Crypt,
                    ", instead of accessing cards, you may play this card for",
                    Mana(0)
                ],
                delegates::on_raid_access_start(requirements::in_hand, |g, s, event| {
                    if event.target == RoomId::Crypt {
                        prompts::push(g, Side::Riftcaller, s);
                    }
                    Ok(())
                }),
            )
            .delegate(this::prompt(|_, s, _, _| {
                show_prompt::with_choices(vec![
                    PromptChoice::new()
                        .custom_label(PromptChoiceLabel::Play)
                        .effect(GameEffect::PlayCardForNoMana(
                            s.card_id(),
                            CardTarget::None,
                            FromZone::Hand,
                            s.initiated_by(),
                        ))
                        .effect(GameEffect::PreventRaidCardAccess)
                        .anchor_card(s.card_id()),
                    PromptChoice::new_continue(),
                ])
            })),
            ActivatedAbility::new(
                costs::sacrifice_and_action(),
                text![text!["Access all cards in the", Crypt], text![GainActions(1)]],
            )
            .delegate(this::on_activated(|g, s, _| {
                custom_access::initiate(
                    g,
                    RoomId::Crypt,
                    InitiatedBy::Ability(s.ability_id()),
                    g.discard_pile(Side::Covenant).map(|c| c.id).collect(),
                )?;

                mutations::gain_action_points(g, s.side(), 1)
            }))
            .build(),
        ],
        config: CardConfig::default(),
    }
}

pub fn a_moments_peace(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::AMomentsPeace,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(5),
        image: assets::riftcaller_card(meta, "a_moments_peace"),
        card_type: CardType::Evocation,
        subtypes: vec![CardSubtype::Charge],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Rare,
        abilities: vec![
            Ability::new_with_delegate(
                text!["The Covenant cannot win the game by scoring points"],
                in_play::can_win_by_scoring_points(|_, s, side, current| {
                    if *side == Side::Covenant {
                        current.disallow(s)
                    } else {
                        current
                    }
                }),
            ),
            Ability::new(text_helpers::named_trigger(
                Dawn,
                text![
                    text![AddPowerCharges(1)],
                    text![
                        "If there are",
                        meta.upgrade(3, 4),
                        "or more",
                        PowerChargeSymbol,
                        ",",
                        Banish,
                        "this card"
                    ],
                ],
            ))
            .delegate(in_play::at_dawn(|g, s, _| {
                mutations::add_power_charges(g, s.card_id(), 1)?;
                if g.card(s.card_id()).counters(CardCounter::PowerCharges) >= s.upgrade(3, 4) {
                    mutations::banish_card(g, s.card_id())?;
                }
                Ok(())
            }))
            .delegate(this::on_leaves_play(|g, _, _| mutations::check_for_score_victory(g))),
        ],
        config: CardConfigBuilder::new()
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles2(12))
                    .scale(1.0)
                    .effect_color(design::YELLOW_900),
            )
            .build(),
    }
}

pub fn vortex_portal(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::VortexPortal,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::riftcaller_card(meta, "vortex_portal"),
        card_type: CardType::Evocation,
        subtypes: vec![CardSubtype::Charge],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![
            Ability::new_with_delegate(
                text!["When the Covenant scores a scheme,", AddPowerCharges(1)],
                in_play::on_covenant_scored_card(|g, s, _| {
                    g.card_mut(s).add_counters(CardCounter::PowerCharges, 1);
                    Ok(())
                }),
            ),
            ActivatedAbility::new(
                costs::power_charges_and_action::<1>(),
                text![text!["Raid the sanctum,", Evading, "all defenders"], text![GainActions(1)]],
            )
            .delegate(this::on_activated(|g, s, _| raids::initiate(g, s, RoomId::Sanctum)))
            .delegate(delegates::on_will_populate_summon_prompt(
                requirements::matching_raid,
                |g, _, _| mutations::evade_current_minion(g),
            ))
            .build(),
        ],
        config: CardConfig::default(),
    }
}

pub fn radiant_intervention(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::RadiantIntervention,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::riftcaller_card(meta, "radiant_intervention"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Uncommon,
        abilities: vec![Ability::new_with_delegate(
            text![TextElement::Activated {
                cost: text![SacrificeCost],
                effect: text![
                    text!["Prevent an ally or artifact from being destroyed"],
                    meta.upgraded_only_text(text!["Draw a card"])
                ]
            }],
            in_play::on_will_destroy_cards(|g, s, _| {
                prompts::push(g, Side::Riftcaller, s);
                Ok(())
            }),
        )
        .delegate(this::prompt(|g, s, _, _| {
            let targets = destroy::all_targets(g);
            show_prompt::with_context_and_choices(
                ButtonPromptContext::SacrificeToPreventDestroyingCard(s.card_id()),
                targets
                    .iter()
                    .filter(|card_id| {
                        let definition = g.card(**card_id).definition();
                        definition.is_ally() || definition.is_artifact()
                    })
                    .map(|card_id| {
                        PromptChoice::new()
                            .effect(GameEffect::SacrificeCard(s.card_id()))
                            .effect(GameEffect::PreventDestroyingCard(*card_id))
                            .effect_optional(s.is_upgraded().then(|| {
                                GameEffect::DrawCards(Side::Riftcaller, 1, s.initiated_by())
                            }))
                            .custom_label(PromptChoiceLabel::Prevent)
                            .anchor_card(*card_id)
                    })
                    .chain(iter::once(PromptChoice::new_continue()))
                    .collect(),
            )
        }))],
        config: CardConfig::default(),
    }
}

pub fn lightcallers_command(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::LightcallersCommand,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(3),
        image: assets::riftcaller_card(meta, "lightcallers_command"),
        card_type: CardType::Evocation,
        subtypes: vec![CardSubtype::Decree],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Uncommon,
        abilities: vec![ActivatedAbility::new(
            costs::sacrifice(),
            meta.upgrade(
                text!["The Covenant cannot summon the outermost minion in any room this turn"],
                text!["The Covenant cannot summon minions this turn"],
            ),
        )
        .delegate(delegates::can_summon(
            requirements::ability_activated_this_turn,
            |g, s, &card_id, flag| {
                if s.is_upgraded() {
                    flag.disallow()
                } else {
                    flag.add_constraint(!flags::is_outermost_defender(g, card_id))
                }
            },
        ))
        .delegate(delegates::status_markers(
            requirements::ability_activated_this_turn,
            |g, s, &card_id, mut markers| {
                if g.card(card_id).definition().is_minion()
                    && g.card(card_id).is_face_down()
                    && (flags::is_outermost_defender(g, card_id) || s.is_upgraded())
                {
                    markers.push(CardStatusMarker {
                        source: s.ability_id(),
                        marker_kind: CardInfoElementKind::NegativeEffect,
                        text: text!["Cannot be summoned this turn"],
                    });
                }
                markers
            },
        ))
        .build()],
        config: CardConfig::default(),
    }
}

pub fn potentiality_storm(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::PotentialityStorm,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::riftcaller_card(meta, "potentiality_storm"),
        card_type: CardType::Evocation,
        subtypes: vec![CardSubtype::Arcane],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Rare,
        abilities: vec![
            abilities::store_mana_on_play::<3>(),
            Ability::new(text_helpers::named_trigger(
                Dawn,
                text![text![TakeMana(1)], text!["Sacrifice and draw a card when empty"]],
            ))
            .delegate(in_play::at_dawn(|g, s, _| {
                visual_effects::show(g, s, s.card_id(), ShowAlert::No);
                mutations::take_stored_mana(g, s.card_id(), 1, OnZeroStored::Ignore)?;
                if g.card(s.card_id()).counters(CardCounter::StoredMana) == 0 {
                    draw_cards::run(g, Side::Riftcaller, 1, s.initiated_by())?;
                    mutations::sacrifice_card(g, s.card_id())?;
                }
                Ok(())
            })),
            Ability::new(text_helpers::named_trigger(
                Dusk,
                text![
                    "If you made",
                    meta.upgrade(3, 2),
                    "successful raids last turn, you may play this card from your discard pile"
                ],
            ))
            .delegate(this::at_dusk(|g, s, _| {
                if g.card(s).position().in_discard_pile()
                    && history::last_turn(g)
                        .filter(|event| matches!(event, HistoryEvent::RaidSuccess(..)))
                        .count()
                        >= s.upgrade(3, 2)
                {
                    visual_effects::show_alert(g, s);
                    prompts::push(g, Side::Riftcaller, s);
                }
                Ok(())
            }))
            .delegate(this::prompt(|_, s, _, _| {
                PlayCardBrowserBuilder::new(s, FromZone::Discard, vec![s.card_id()]).build()
            })),
        ],
        config: CardConfigBuilder::new()
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles2(12))
                    .scale(1.5)
                    .sound(SoundEffect::WaterMagic("RPG3_WaterMagicEpic_Cast02"))
                    .effect_color(design::BLUE_500),
            )
            .build(),
    }
}
