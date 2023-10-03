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

use card_helpers::updates::Updates;
use card_helpers::{abilities, costs, delegates, raids, requirements, show_prompt, text, this};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardPosition;
use game_data::game_actions::{CardTarget, PromptChoice, PromptChoiceLabel, PromptContext};
use game_data::game_effect::GameEffect;
use game_data::primitives::{CardSubtype, CardType, GameObjectId, Rarity, RoomId, School, Side};
use game_data::special_effects::{Projectile, SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextToken::*;
use rules::{mutations, CardDefinitionExt};

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
            Some(abilities::standard(
                text!["Play an artifact in your discard pile"],
                this::on_play(|g, s, _| {
                    let cards = g
                        .discard_pile(s.side())
                        .filter(|c| c.definition().card_type == CardType::Artifact)
                        .map(|c| c.id)
                        .collect::<Vec<_>>();

                    Updates::new(g)
                        .timed_effect(
                            GameObjectId::DiscardPile(Side::Champion),
                            TimedEffectData::new(TimedEffect::MagicCircles1(2))
                                .scale(2.0)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagic_Buff03_P1"))
                                .effect_color(design::YELLOW_900),
                        )
                        .card_movement_effects(Projectile::Projectiles1(3), &cards)
                        .apply();

                    show_prompt::play_card_browser(
                        g,
                        s,
                        cards,
                        PromptContext::PlayFromDiscard(CardType::Artifact),
                    )
                }),
            )),
            abilities::when_upgraded(
                meta,
                abilities::standard(
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
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: text!["Raid the", Sanctum, ", accessing", meta.upgrade(2, 3), "additional cards"],
            delegates: vec![
                this::on_play(|g, s, _| {
                    Updates::new(g)
                        .timed_effect(
                            GameObjectId::Character(Side::Overlord),
                            TimedEffectData::new(TimedEffect::MagicCircles1(1))
                                .scale(2.0)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagic_Cast01"))
                                .effect_color(design::YELLOW_900),
                        )
                        .apply();

                    raids::initiate(g, s, CardTarget::Room(RoomId::Sanctum))
                }),
                delegates::sanctum_access_count(requirements::matching_raid, |_, s, _, v| {
                    v + s.upgrade(2, 3)
                }),
            ],
        }],
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
        abilities: vec![abilities::standard(
            text![
                text!["Remove a", Curse],
                meta.upgrade(
                    text!["You may pay", Mana(1), "to return this card to your hand"],
                    text!["Return this card to your hand"]
                ),
            ],
            this::on_play(|g, s, _| {
                mutations::remove_curses(g, 1)?;

                Updates::new(g)
                    .timed_effect(
                        GameObjectId::Character(Side::Champion),
                        TimedEffectData::new(TimedEffect::MagicCircles1(3))
                            .scale(2.0)
                            .sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_Heal02"))
                            .effect_color(design::YELLOW_900),
                    )
                    .apply();

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