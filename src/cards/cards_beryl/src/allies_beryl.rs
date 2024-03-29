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

use card_definition_data::ability_data::{Ability, ActivatedAbility};
use card_definition_data::card_definition::CardDefinition;
use card_definition_data::cards::CardDefinitionExt;
use card_helpers::{costs, history, in_play, show_prompt, text, text_helpers, this};
use core_data::game_primitives::{
    CardSubtype, CardType, GameObjectId, Rarity, RoomId, School, Side,
};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_configuration::{CardConfig, CardConfigBuilder, TargetRequirement};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardCounter;
use game_data::custom_card_state::CustomCardState;
use game_data::delegate_data::{CardInfoElementKind, CardStatusMarker};
use game_data::game_actions::{ButtonPromptContext, CardTarget};
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::prompt_data::{PromptChoice, PromptData};
use game_data::raid_data::RaidJumpRequest;
use game_data::special_effects::{SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextElement;
use game_data::text::TextToken::*;
use game_data::utils;
use rules::mutations::OnZeroStored;
use rules::visual_effects::{ShowAlert, VisualEffects};
use rules::{
    curses, damage, draw_cards, mana, mutations, prompts, queries, visual_effects, wounds,
};

pub fn astrian_oracle(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::AstrianOracle,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(5, 8)),
        image: assets::riftcaller_card(meta, "astrian_oracle"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Noble],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new(text![
            "When you raid the",
            Sanctum,
            ", access",
            meta.upgrade("an additional card", "two additional cards")
        ])
        .delegate(in_play::on_query_sanctum_access_count(|_, s, _, current| {
            current + s.upgrade(1, 2)
        }))
        .delegate(in_play::on_sanctum_access_start(|g, s, _| {
            VisualEffects::new()
                .timed_effect(
                    GameObjectId::CardId(s.card_id()),
                    TimedEffectData::new(TimedEffect::MagicCircles2(15))
                        .scale(2.0)
                        .effect_color(design::YELLOW_900),
                )
                .apply(g);
            Ok(())
        }))],
        config: CardConfig::default(),
    }
}

pub fn resplendent_channeler(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ResplendentChanneler,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(3),
        image: assets::riftcaller_card(meta, "resplendent_channeler"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Cleric],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![
                "The first time you access the sanctum each turn, draw a card and gain",
                Mana(meta.upgrade(1, 3))
            ],
            in_play::on_sanctum_access_start(|g, s, _| {
                if history::rooms_accessed_this_turn(g).all(|r| r != RoomId::Sanctum) {
                    VisualEffects::new()
                        .timed_effect(
                            GameObjectId::CardId(s.card_id()),
                            TimedEffectData::new(TimedEffect::MagicCircles2(14))
                                .scale(2.0)
                                .sound(SoundEffect::LightMagic(
                                    "RPG3_LightMagicMisc_AttackMissed04",
                                ))
                                .effect_color(design::YELLOW_900),
                        )
                        .ability_alert(s)
                        .apply(g);

                    mana::gain(g, s.side(), s.upgrade(1, 3));
                    draw_cards::run(g, s.side(), 1, s.initiated_by())?;
                }
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn stalwart_protector(meta: CardMetadata) -> CardDefinition {
    fn update(game: &mut GameState) {
        VisualEffects::new()
            .timed_effect(
                GameObjectId::Character(Side::Riftcaller),
                TimedEffectData::new(TimedEffect::MagicCircles1(7))
                    .scale(2.0)
                    .sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_HealingWing_P1"))
                    .effect_color(design::YELLOW_900),
            )
            .apply(game);
    }

    CardDefinition {
        name: CardName::StalwartProtector,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(1, 0)),
        image: assets::riftcaller_card(meta, "stalwart_protector"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Warrior],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text![TextElement::Activated {
                    cost: text![SacrificeCost],
                    effect: text!["Prevent receiving a", Curse]
                }],
                in_play::on_will_receive_curses(|g, s, _| {
                    prompts::push(g, Side::Riftcaller, s);
                    Ok(())
                }),
            )
            .delegate(this::prompt(|_, s, _, _| {
                show_prompt::with_context_and_choices(
                    ButtonPromptContext::SacrificeToPreventCurses(s.card_id(), 1),
                    vec![
                        PromptChoice::new()
                            .effect(GameEffect::SacrificeCard(s.card_id()))
                            .effect(GameEffect::PreventCurses(1)),
                        PromptChoice::new().effect(GameEffect::Continue),
                    ],
                )
            })),
            ActivatedAbility::new(costs::sacrifice(), text!["Remove a curse"])
                .delegate(this::can_activate(|g, _, _, flag| {
                    flag.add_constraint(curses::is_riftcaller_cursed(g))
                }))
                .delegate(this::on_activated(|g, _, _| {
                    update(g);
                    curses::remove_curses(g, 1)
                }))
                .build(),
        ],
        config: CardConfig::default(),
    }
}

pub fn dawnwarden(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Dawnwarden,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::riftcaller_card(meta, "dawnwarden"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Cleric],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text![
                    "When an artifact is put into your discard pile,",
                    StoreMana(meta.upgrade(2, 3))
                ],
                in_play::on_card_moved_to_discard_pile(|g, s, card_id| {
                    if g.card(*card_id).definition().card_type == CardType::Artifact {
                        VisualEffects::new()
                            .timed_effect(
                                GameObjectId::CardId(s.card_id()),
                                TimedEffectData::new(TimedEffect::MagicCircles2(13))
                                    .scale(2.0)
                                    .sound(SoundEffect::LightMagic(
                                        "RPG3_LightMagicEpic_Transform02",
                                    ))
                                    .effect_color(design::YELLOW_900),
                            )
                            .apply(g);

                        mutations::add_stored_mana(g, s.card_id(), s.upgrade(2, 3));
                    }
                    Ok(())
                }),
            ),
            ActivatedAbility::new(costs::actions(1), text!["Take all stored mana"])
                .delegate(this::can_activate(|g, s, _, flag| {
                    flag.add_constraint(g.card(s.card_id()).counters(CardCounter::StoredMana) > 0)
                }))
                .delegate(this::on_activated(|g, s, _| {
                    mutations::take_stored_mana(
                        g,
                        s.card_id(),
                        g.card(s.card_id()).counters(CardCounter::StoredMana),
                        OnZeroStored::Ignore,
                    )?;
                    Ok(())
                }))
                .build(),
        ],
        config: CardConfig::default(),
    }
}

pub fn spellcraft_ritualist(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SpellcraftRitualist,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(2),
        image: assets::riftcaller_card(meta, "spellcraft_ritualist"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Mage],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text![TextElement::NamedTrigger(Play, text!["Take a", Wound])],
                this::on_played(|g, s, _| wounds::give(g, s.ability_id(), 1)),
            ),
            Ability::new_with_delegate(
                text!["Your spells cost", Mana(meta.upgrade(1, 2)), "less"],
                in_play::on_query_mana_cost(|g, s, card_id, cost| {
                    if g.card(*card_id).definition().card_type.is_spell()
                        && card_id.side == s.side()
                    {
                        cost.map(|c| c.saturating_sub(s.upgrade(1, 2)))
                    } else {
                        cost
                    }
                }),
            ),
        ],
        config: CardConfig::default(),
    }
}

pub fn blue_warden(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::BlueWarden,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(3, 2)),
        image: assets::riftcaller_card(meta, "blue_warden"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Warrior],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Uncommon,
        abilities: vec![
            ActivatedAbility::new(costs::sacrifice(), text!["Draw", meta.upgrade(3, 4), "cards"])
                .delegate(this::on_activated(|g, s, _| {
                    draw_cards::run(g, s.side(), s.upgrade(3, 4), s.initiated_by())
                }))
                .build(),
            Ability::new(text!["You may activate abilities after being damaged"])
                .delegate(in_play::on_damage(|g, s, _| {
                    prompts::push(g, Side::Riftcaller, s);
                    Ok(())
                }))
                .delegate(this::prompt(|_, _, _, _| show_prompt::priority_window())),
        ],
        config: CardConfig::default(),
    }
}

pub fn noble_martyr(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::NobleMartyr,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(2, 0)),
        image: assets::riftcaller_card(meta, "noble_martyr"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Cleric],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Uncommon,
        abilities: vec![
            Ability::new(text_helpers::named_trigger(
                Play,
                text!["Choose a minion in target room with", 2, "or fewer", ShieldPoints],
            ))
            .delegate(this::on_played(|g, s, played| {
                prompts::push_with_data(g, s.side(), s, PromptData::CardPlay(*played));
                Ok(())
            }))
            .delegate(this::prompt(|g, s, source, _| {
                let PromptData::CardPlay(played) = source.data else {
                    return None;
                };
                let CardTarget::Room(room_id) = played.target else {
                    return None;
                };
                show_prompt::with_choices(
                    g.defenders_unordered(room_id)
                        .filter(|card| queries::shield(g, card.id, None) <= 2)
                        .map(|card| {
                            PromptChoice::new()
                                .effect(GameEffect::AppendCustomCardState(
                                    s.card_id(),
                                    CustomCardState::TargetCard {
                                        target_card: card.id,
                                        play_id: played.card_play_id,
                                    },
                                ))
                                .anchor_card(card.id)
                        })
                        .collect(),
                )
            }))
            .delegate(in_play::on_query_card_status_markers(
                |g, s, card_id, mut markers| {
                    if g.card(s).is_last_target(*card_id) {
                        markers.push(CardStatusMarker {
                            source: s.ability_id(),
                            marker_kind: CardInfoElementKind::NegativeEffect,
                            text: text!["Can defeat by sacrificing"],
                        });
                    }
                    markers
                },
            )),
            ActivatedAbility::new(costs::sacrifice(), text!["Defeat the chosen minion"])
                .delegate(this::on_activated(|g, _, _| {
                    mutations::apply_raid_jump(g, RaidJumpRequest::DefeatCurrentMinion);
                    Ok(())
                }))
                .delegate(this::can_activate(|g, s, _, flag| {
                    flag.add_constraint(utils::is_true(|| {
                        Some(g.card(s).is_last_target(g.current_raid_defender()?))
                    }))
                }))
                .build(),
        ],
        config: CardConfigBuilder::new()
            .custom_targeting(TargetRequirement::TargetRoom(|g, _, room_id| {
                g.defenders_unordered(room_id).any(|card| queries::shield(g, card.id, None) <= 2)
            }))
            .build(),
    }
}

pub fn rift_adept(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::RiftAdept,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(5, 3)),
        image: assets::riftcaller_card(meta, "rift_adept"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Mage],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Uncommon,
        abilities: vec![Ability::new(text![
            "When you finish accessing the",
            Crypt,
            ", access the",
            Vault
        ])
        .delegate(in_play::on_raid_access_end(|g, s, event| {
            if event.target == RoomId::Crypt {
                visual_effects::show(g, s, s.card_id(), ShowAlert::Yes);
                mutations::apply_raid_jump(
                    g,
                    RaidJumpRequest::AddAdditionalTargetRoom(RoomId::Vault),
                );
            }
            Ok(())
        }))],
        config: CardConfigBuilder::new()
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles2(17))
                    .scale(2.0)
                    .effect_color(design::BLUE_500)
                    .sound(SoundEffect::WaterMagic("RPG3_WaterMagic3_TimeStop_Full")),
            )
            .build(),
    }
}

pub fn phalanx_guardian(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::PhalanxGuardian,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::riftcaller_card(meta, "phalanx_guardian"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Warrior],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Uncommon,
        abilities: vec![Ability::new(text![
            "Prevent the first",
            meta.upgrade(1, 2),
            "damage you would take each turn"
        ])
        .delegate(in_play::on_will_deal_damage(|g, s, _| {
            let turn = g.info.turn;
            let Some(play_id) = g.card(s).last_card_play_id else {
                return Ok(());
            };
            if utils::is_true(|| Some(damage::incoming_amount(g)? > 0))
                && !g.card(s).custom_state.in_play_ability_triggered_for_turn(turn, play_id)
                && history::counters(g, Side::Riftcaller).damage_received == 0
            {
                visual_effects::show(g, s, s.card_id(), ShowAlert::Yes);
                g.card_mut(s)
                    .custom_state
                    .push(CustomCardState::InPlayAbilityTriggeredForTurn { turn, play_id });
                damage::prevent(g, s.upgrade(1, 2));
            }
            Ok(())
        }))],
        config: CardConfigBuilder::new()
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles2(19))
                    .scale(1.5)
                    .effect_color(design::YELLOW_900)
                    .sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_Heal01")),
            )
            .build(),
    }
}
