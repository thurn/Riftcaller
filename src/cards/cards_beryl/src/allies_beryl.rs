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

use core_data::game_primitives::{
    CardSubtype, CardType, GameObjectId, Rarity, RoomId, School, Side,
};
use game_data::card_definition::{Ability, ActivatedAbility, CardConfig, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardCounter;
use game_data::game_actions::{ButtonPromptContext, PromptChoice};
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::special_effects::{SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextElement;
use game_data::text::TextToken::*;

use card_helpers::{costs, history, in_play, show_prompt, text, this};
use card_helpers::effects::Effects;
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use rules::{CardDefinitionExt, curses, mana, mutations, wounds};
use rules::mutations::OnZeroStored;

pub fn astrian_oracle(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::AstrianOracle,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(5, 8)),
        image: assets::champion_card(meta, "astrian_oracle"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Noble],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new(text![
            "When you raid the",
            Sanctum,
            meta.upgrade("an additional card", "two additional cards")
        ])
        .delegate(in_play::on_query_sanctum_access_count(|_, s, _, current| {
            current + s.upgrade(1, 2)
        }))
        .delegate(in_play::on_sanctum_access_start(|g, s, _| {
            Effects::new()
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
        image: assets::champion_card(meta, "resplendent_channeler"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Cleric],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![
                "The first time you access the sanctum each turn, draw a card and gain",
                Mana(meta.upgrade(1, 3))
            ],
            in_play::on_sanctum_access_start(|g, s, _| {
                if history::rooms_accessed_this_turn(g).all(|r| r != RoomId::Sanctum) {
                    Effects::new()
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
                    mutations::draw_cards(g, s.side(), 1)?;
                }
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn stalwart_protector(meta: CardMetadata) -> CardDefinition {
    fn update(game: &mut GameState) {
        Effects::new()
            .timed_effect(
                GameObjectId::Character(Side::Champion),
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
        image: assets::champion_card(meta, "stalwart_protector"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Warrior],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text![TextElement::Activated {
                    cost: text![SacrificeCost],
                    effect: text!["Prevent receiving a", Curse]
                }],
                in_play::on_will_receive_curses(|g, s, _| {
                    show_prompt::with_context_and_choices(
                        g,
                        s,
                        ButtonPromptContext::SacrificeToPreventCurses(s.card_id(), 1),
                        vec![
                            PromptChoice::new()
                                .effect(GameEffect::SacrificeCard(s.card_id()))
                                .effect(GameEffect::PreventCurses(1)),
                            PromptChoice::new().effect(GameEffect::Continue),
                        ],
                    );
                    Ok(())
                }),
            ),
            ActivatedAbility::new(costs::sacrifice(), text!["Remove a curse"])
                .delegate(this::can_activate(|g, _, _, flag| {
                    flag.add_constraint(g.champion.curses > 0)
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
        image: assets::champion_card(meta, "dawnwarden"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Cleric],
        side: Side::Champion,
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
                        Effects::new()
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
        image: assets::champion_card(meta, "spellcraft_ritualist"),
        card_type: CardType::Ally,
        subtypes: vec![CardSubtype::Mage],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text![TextElement::NamedTrigger(Play, text!["Take a", Wound])],
                this::on_played(|g, s, _| wounds::give_wounds(g, s, 1)),
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
