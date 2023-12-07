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

use card_helpers::play_card_browser_builder::PlayCardBrowserBuilder;
use card_helpers::{
    costs, delegates, in_play, play_card_browser_builder, raids, requirements, show_prompt, text,
    text_helpers, this,
};
use core_data::game_primitives::{
    CardId, CardSubtype, CardType, GameObjectId, InitiatedBy, Rarity, RoomId, School, Side,
};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::animation_tracker::GameAnimation;
use game_data::card_definition::{
    Ability, ActivatedAbility, CardConfig, CardConfigBuilder, CardDefinition, Cost,
    TargetRequirement,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::{BanishedByCard, CardCounter, CardPosition};
use game_data::game_actions::{ButtonPromptContext, CardTarget};
use game_data::game_effect::GameEffect;
use game_data::prompt_data::{PromptChoice, PromptChoiceLabel, UnplayedAction};
use game_data::special_effects::{Projectile, SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextElement;
use game_data::text::TextToken::*;
use raid_state::{custom_access, InitiateRaidOptions};
use rules::visual_effects::VisualEffects;
use rules::{curses, draw_cards, flags, mana, mutations, CardDefinitionExt};

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
            costs::sacrifice_for_action(),
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
                    show_prompt::with_context_and_choices(
                        g,
                        s,
                        ButtonPromptContext::SacrificeToPreventDamage(s.card_id(), s.upgrade(2, 5)),
                        vec![
                            PromptChoice::new()
                                .effect(GameEffect::SacrificeCard(s.card_id()))
                                .effect(GameEffect::PreventDamage(s.upgrade(2, 5))),
                            PromptChoice::new().effect(GameEffect::Continue),
                        ],
                    );
                }
                Ok(())
            }),
        )],
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
        subtypes: vec![CardSubtype::Mystic],
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
                        show_prompt::priority_window(g, s);
                    }
                    Ok(())
                }))
                .delegate(in_play::on_damage(|g, s, _| {
                    if g.card(s.card_id()).counters(CardCounter::PowerCharges) > 0 {
                        show_prompt::priority_window(g, s);
                    }
                    Ok(())
                })),
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
                let cards = mutations::realize_top_of_deck(g, s.side(), 3)?;
                mutations::set_cards_visible_to(g, &cards, s.side(), true);
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

                play_card_browser_builder::show(
                    g,
                    PlayCardBrowserBuilder::new(s, permanents)
                        .movement_effect(Projectile::Projectiles1(2))
                        .unplayed_action(UnplayedAction::Discard),
                )
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
                        show_prompt::with_choices(
                            g,
                            s,
                            vec![
                                PromptChoice::new()
                                    .custom_label(PromptChoiceLabel::Play)
                                    .effect(GameEffect::PlayCardForNoMana(
                                        s.card_id(),
                                        CardTarget::None,
                                        s.initiated_by(),
                                    ))
                                    .effect(GameEffect::PreventRaidCardAccess)
                                    .anchor_card(s.card_id()),
                                PromptChoice::new_continue(),
                            ],
                        )
                    }
                    Ok(())
                }),
            ),
            ActivatedAbility::new(
                costs::sacrifice_for_action(),
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
        subtypes: vec![],
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
            .choice_effect(
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
        subtypes: vec![],
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
                Cost {
                    mana: None,
                    actions: 1,
                    custom_cost: costs::power_charges_custom_cost::<1>(),
                },
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
