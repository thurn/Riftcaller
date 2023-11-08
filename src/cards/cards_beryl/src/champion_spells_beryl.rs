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

use card_helpers::effects::Effects;
use card_helpers::{
    abilities, costs, delegates, history, raids, requirements, show_prompt, text, this,
};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_definition::{Ability, CardConfig, CardConfigBuilder, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardPosition;
use game_data::game_actions::{
    CardTarget, PromptChoice, PromptChoiceLabel, PromptContext, UnplayedAction,
};
use game_data::game_effect::GameEffect;
use game_data::game_state::{GameState, RaidJumpRequest};
use game_data::primitives::{
    AbilityId, CardSubtype, CardType, GameObjectId, Rarity, RoomId, School, Side,
};
use game_data::special_effects::{Projectile, SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextToken::*;
use rules::{curses, mutations, CardDefinitionExt};

pub fn restoration(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Restoration,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::champion_card(meta, "restoration"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![CardSubtype::Conjuration],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: abilities::some(vec![
            Some(abilities::silent_can_play(|g, _, _, current| {
                current.with_override(
                    g.discard_pile(Side::Champion).any(|c| c.definition().is_artifact()),
                )
            })),
            Some(Ability::new_with_delegate(
                text!["Play an artifact in your discard pile"],
                this::on_played(|g, s, _| {
                    let cards = g
                        .discard_pile(s.side())
                        .filter(|c| c.definition().is_artifact())
                        .map(|c| c.id)
                        .collect::<Vec<_>>();

                    Effects::new()
                        .timed_effect(
                            GameObjectId::DiscardPile(Side::Champion),
                            TimedEffectData::new(TimedEffect::MagicCircles1(2))
                                .scale(2.0)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagic_Buff03_P1"))
                                .effect_color(design::YELLOW_900),
                        )
                        .card_movement_effects(Projectile::Projectiles1(3), &cards)
                        .apply(g);

                    show_prompt::play_card_browser(
                        g,
                        s,
                        cards,
                        PromptContext::PlayFromDiscard(CardType::Artifact),
                        UnplayedAction::None,
                    )
                }),
            )),
            abilities::when_upgraded(
                meta,
                Ability::new_with_delegate(
                    text!["Reduce its cost by", Mana(2)],
                    delegates::mana_cost(requirements::matching_play_browser, |_, _, _, cost| {
                        cost.map(|c| c.saturating_sub(2))
                    }),
                ),
            ),
        ]),
        config: CardConfig::default(),
    }
}

pub fn strike_the_heart(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::StrikeTheHeart,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(3),
        image: assets::champion_card(meta, "strike_the_heart"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![CardSubtype::Raid],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new(text![
            "Raid the",
            Sanctum,
            ", accessing",
            meta.upgrade(2, 3),
            "additional cards"
        ])
        .delegate(this::on_played(|g, s, _| {
            Effects::new()
                .timed_effect(
                    GameObjectId::Character(Side::Overlord),
                    TimedEffectData::new(TimedEffect::MagicCircles1(1))
                        .scale(2.0)
                        .sound(SoundEffect::LightMagic("RPG3_LightMagic_Cast01"))
                        .effect_color(design::YELLOW_900),
                )
                .apply(g);

            raids::initiate(g, s, CardTarget::Room(RoomId::Sanctum))
        }))
        .delegate(delegates::sanctum_access_count(requirements::matching_raid, |_, s, _, v| {
            v + s.upgrade(2, 3)
        }))],
        config: CardConfig::default(),
    }
}

pub fn enduring_radiance(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EnduringRadiance,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::champion_card(meta, "enduring_radiance"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            text![
                text!["Remove a", Curse],
                meta.upgrade(
                    text!["You may pay", Mana(1), "to return this card to your hand"],
                    text!["Return this card to your hand"]
                ),
            ],
            this::on_played(|g, s, _| {
                curses::remove_curses(g, 1)?;

                Effects::new()
                    .timed_effect(
                        GameObjectId::Character(Side::Champion),
                        TimedEffectData::new(TimedEffect::MagicCircles1(3))
                            .scale(2.0)
                            .sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_Heal02"))
                            .effect_color(design::YELLOW_900),
                    )
                    .apply(g);

                if s.is_upgraded() {
                    mutations::move_card(g, s.card_id(), CardPosition::Hand(s.side()))?;
                } else {
                    show_prompt::with_choices(
                        g,
                        s,
                        vec![
                            PromptChoice::new()
                                .effect(GameEffect::MoveCard(
                                    s.card_id(),
                                    CardPosition::Hand(s.side()),
                                ))
                                .effect(GameEffect::LoseMana(s.side(), 1))
                                .custom_label(PromptChoiceLabel::Return(1)),
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

pub fn sift_the_sands(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SiftTheSands,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::champion_card(meta, "sift_the_sands"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![CardSubtype::Conjuration],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new(text![
            text!["Look at the top", meta.upgrade(4, 6), "cards of your deck"],
            text!["You may play one of them, paying", Mana(3), "less"],
            text!["Discard the rest"]
        ])
        .delegate(this::on_played(|g, s, _| {
            let cards = mutations::realize_top_of_deck(g, s.side(), s.upgrade(4, 6))?;
            for card in &cards {
                mutations::set_revealed_to(g, *card, s.side(), true);
            }

            Effects::new()
                .timed_effect(
                    GameObjectId::Deck(Side::Champion),
                    TimedEffectData::new(TimedEffect::MagicCircles1(4))
                        .scale(2.0)
                        .sound(SoundEffect::LightMagic("RPG3_LightMagic4_P1_Cast"))
                        .effect_color(design::YELLOW_900),
                )
                .card_movement_effects(Projectile::Projectiles1(3), &cards)
                .apply(g);

            show_prompt::play_card_browser(
                g,
                s,
                cards,
                PromptContext::PlayACard,
                UnplayedAction::Discard,
            )?;

            Ok(())
        }))
        .delegate(delegates::mana_cost(requirements::matching_play_browser, |_, _, _, cost| {
            cost.map(|c| c.saturating_sub(3))
        }))],
        config: CardConfig::default(),
    }
}

pub fn holy_aura(meta: CardMetadata) -> CardDefinition {
    fn update(game: &mut GameState, alert: Option<AbilityId>) {
        Effects::new()
            .timed_effect(
                GameObjectId::Deck(Side::Champion),
                TimedEffectData::new(TimedEffect::MagicCircles1(5))
                    .scale(4.0)
                    .sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_HealingWing_P1"))
                    .effect_color(design::YELLOW_900),
            )
            .optional_ability_alert(alert)
            .apply(game);
    }

    CardDefinition {
        name: CardName::HolyAura,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::champion_card(meta, "holy_aura"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text!["Draw", meta.upgrade(3, 4), "cards"],
                this::on_played(|g, s, _| {
                    update(g, None);
                    mutations::draw_cards(g, s.side(), s.upgrade(3, 4))?;
                    Ok(())
                }),
            ),
            Ability::new_with_delegate(
                text!["If this card is discarded, draw", meta.upgrade(2, 3), "cards"],
                this::on_discarded(|g, s, _| {
                    update(g, Some(s.ability_id()));
                    mutations::draw_cards(g, s.side(), s.upgrade(2, 3))?;
                    Ok(())
                }),
            ),
        ],
        config: CardConfig::default(),
    }
}

pub fn voidstep(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Voidstep,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(2, 0)),
        image: assets::champion_card(meta, "voidstep"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![CardSubtype::Raid],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![Ability::new(text![
            text!["Raid target room"],
            text![Evade, "the first minion you encounter"]
        ])
        .delegate(this::on_played(|g, s, play_card| {
            Effects::new()
                .timed_effect(
                    GameObjectId::Character(Side::Champion),
                    TimedEffectData::new(TimedEffect::MagicCircles1(2))
                        .scale(2.0)
                        .sound(SoundEffect::WaterMagic("RPG3_WaterMagic3_P1_Castv2"))
                        .effect_color(design::BLUE_500),
                )
                .apply(g);

            raids::initiate(g, s, play_card.target)
        }))
        .delegate(delegates::on_minion_approached(
            requirements::matching_raid,
            |g, _, minion| {
                if history::minions_approached_this_turn(g)
                    .filter(|event| event.raid_id == minion.raid_id)
                    .next()
                    .is_none()
                {
                    mutations::apply_raid_jump(g, RaidJumpRequest::EvadeCurrentMinion);
                }
                Ok(())
            },
        ))],
        config: CardConfigBuilder::new().custom_targeting(requirements::any_raid_target()).build(),
    }
}

pub fn keensight(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Keensight,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::champion_card(meta, "keensight"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![CardSubtype::Raid],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![Ability::new(text![
            text!["Reveal the card occupying target room"],
            text!["You may raid that room"]
        ])
        .delegate(this::on_played(|g, s, play_card| {
            let target = play_card.target.room_id()?;
            Effects::new()
                .timed_effect(
                    GameObjectId::Character(s.side()),
                    TimedEffectData::new(TimedEffect::MagicCircles1(2))
                        .scale(2.0)
                        .sound(SoundEffect::WaterMagic("RPG3_WaterMagic3_P1_Castv2"))
                        .effect_color(design::BLUE_500),
                )
                .apply(g);

            let occupants = g.occupants(target).map(|c| c.id).collect::<Vec<_>>();
            for occupant in occupants {
                mutations::set_revealed_to(g, occupant, s.side(), true);
            }

            show_prompt::with_choices(
                g,
                s,
                vec![
                    PromptChoice::new().effect(GameEffect::InitiateRaid(target, s.ability_id())),
                    PromptChoice::new().effect(GameEffect::Continue),
                ],
            );

            Ok(())
        }))],
        config: CardConfigBuilder::new()
            .custom_targeting(requirements::any_outer_room_raid_target())
            .build(),
    }
}

pub fn ethereal_incursion(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EtherealIncursion,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(2, 0)),
        image: assets::champion_card(meta, "ethereal_incursion"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![CardSubtype::Raid],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![Ability::new(text![
            text!["Raid target room"],
            text!["When this raid ends,", Unsummon, "all minions summoned during the raid"]
        ])
        .delegate(this::on_played(|g, s, play_card| raids::initiate(g, s, play_card.target)))
        .delegate(delegates::on_raid_end(requirements::matching_raid, |g, s, outcome| {
            Effects::new().ability_alert(s).apply(g);

            let minions = history::minions_summoned_this_turn(g)
                .filter(|event| event.raid_id == outcome.raid_id)
                .map(|event| event.data)
                .collect::<Vec<_>>();
            for minion in minions {
                mutations::unsummon_minion(g, minion)?;
            }

            Ok(())
        }))],
        config: CardConfigBuilder::new().custom_targeting(requirements::any_raid_target()).build(),
    }
}

pub fn time_stop(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TimeStop,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::champion_card(meta, "time_stop"),
        card_type: CardType::ChampionSpell,
        subtypes: vec![CardSubtype::Raid],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::play_as_first_action(),
            Ability::new_with_delegate(
                text![text![GainActions(meta.upgrade(1, 2))], text!["Raid target room"]],
                this::on_played(|g, s, played| {
                    mutations::gain_action_points(g, s.side(), s.upgrade(1, 2))?;
                    raids::initiate(g, s, played.target)
                }),
            ),
        ],
        config: CardConfigBuilder::new().custom_targeting(requirements::any_raid_target()).build(),
    }
}
