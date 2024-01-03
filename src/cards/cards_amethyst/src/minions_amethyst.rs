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

//! Card definitions for the Minion card type

use assets::rexard_images;
use assets::rexard_images::RexardPack;
use card_helpers::text_helpers::named_trigger;
use card_helpers::this::combat;
use card_helpers::{combat_abilities, *};
use core_data::game_primitives::{CardType, InitiatedBy, Rarity, School, Side};
use game_data::card_definition::{
    Ability, AbilityType, CardConfigBuilder, CardDefinition, Resonance,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardPosition;
use game_data::delegate_data::{EventDelegate, GameDelegate, RaidOutcome};
use game_data::game_effect::GameEffect;
use game_data::prompt_data::PromptChoice;
use rules::mana::ManaPurpose;
use rules::{damage, end_raid, mana, prompts, queries};

pub fn time_golem(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TimeGolem,
        sets: vec![CardSetName::Amethyst],
        cost: cost(2),
        image: rexard_images::get(RexardPack::MonstersAvatars, "10"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            named_trigger(
                Encounter,
                text!["End the raid unless the Riftcaller pays", Mana(5), "or", Actions(2)],
            ),
            on_encountered(|g, s, _| {
                prompts::push(g, Side::Riftcaller, s);
                Ok(())
            }),
        )
        .delegate(this::prompt(|_, s, _, _| {
            show_prompt::with_choices(vec![
                PromptChoice::new().effect(GameEffect::EndRaid(s.ability_id())),
                PromptChoice::new().effect(GameEffect::ManaCost(
                    Side::Riftcaller,
                    5,
                    s.initiated_by(),
                )),
                PromptChoice::new().effect(GameEffect::ActionCost(Side::Riftcaller, 2)),
            ])
        }))],
        config: CardConfigBuilder::new().health(3).resonance(Resonance::infernal()).build(),
    }
}

pub fn shadow_lurker(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ShadowLurker,
        sets: vec![CardSetName::Amethyst],
        cost: cost(3),
        image: rexard_images::get(RexardPack::MonstersAvatars, "80"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text!["While this minion is in an", OuterRoom, "it has", Plus(2), Health],
                on_calculate_health(|g, s, _, current| match g.card(s.card_id()).position() {
                    CardPosition::Room(_, room_id, _) if !room_id.is_inner_room() => current + 2,
                    _ => current,
                }),
            ),
            combat_abilities::end_raid(),
        ],
        config: CardConfigBuilder::new().health(2).shield(1).resonance(Resonance::astral()).build(),
    }
}

pub fn sphinx_of_winters_breath(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SphinxOfWintersBreath,
        sets: vec![CardSetName::Amethyst],
        cost: cost(2),
        image: rexard_images::get(RexardPack::MonstersAvatars, "17"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Standard,
            text: named_trigger(
                Combat,
                text![
                    text![DealDamage(1)],
                    text!["If a card with an odd mana cost is discarded, end the raid"]
                ],
            ),
            delegates: vec![
                combat(|g, s, _| damage::deal(g, s, 1)),
                GameDelegate::DealtDamage(EventDelegate {
                    requirement: |g, s, data| {
                        let discarded = damage::discarded_to_current_event(g);
                        s.ability_id() == data.source
                            && discarded.iter().any(|card_id| {
                                queries::mana_cost(g, *card_id).unwrap_or(0) % 2 != 0
                            })
                    },
                    mutation: |g, s, _| {
                        end_raid::run(g, InitiatedBy::Ability(s.ability_id()), RaidOutcome::Failure)
                    },
                }),
            ],
        }],
        config: CardConfigBuilder::new().health(3).shield(1).resonance(Resonance::mortal()).build(),
    }
}

pub fn bridge_troll(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::BridgeTroll,
        sets: vec![CardSetName::Amethyst],
        cost: cost(2),
        image: rexard_images::get(RexardPack::MonstersAvatars, "29"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new_with_delegate(
            named_trigger(
                Combat,
                text![
                    text!["The Riftcaller loses", Mana(3)],
                    text!["If they have", Mana(6), "or less, end the raid"]
                ],
            ),
            combat(|g, s, _| {
                mana::lose_upto(
                    g,
                    Side::Riftcaller,
                    s.initiated_by(),
                    ManaPurpose::PayForTriggeredAbility,
                    3,
                )?;
                if mana::get(g, Side::Riftcaller, ManaPurpose::BaseMana) <= 6 {
                    end_raid::run(g, InitiatedBy::Ability(s.ability_id()), RaidOutcome::Failure)?;
                }
                Ok(())
            }),
        )],
        config: CardConfigBuilder::new().health(0).shield(2).resonance(Resonance::mortal()).build(),
    }
}

pub fn stormcaller(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Stormcaller,
        sets: vec![CardSetName::Amethyst],
        cost: cost(4),
        image: rexard_images::get(RexardPack::MonstersAvatars, "53"),
        card_type: CardType::Minion,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability::new(named_trigger(
            Combat,
            text![
                text![DealDamage(2)],
                text!["The Riftcaller must end the raid or take 2 more damage"]
            ],
        ))
        .delegate(combat(|g, s, _| {
            damage::deal(g, s, 2)?;
            prompts::push(g, Side::Riftcaller, s);
            Ok(())
        }))
        .delegate(this::prompt(|_, s, _, _| {
            show_prompt::with_choices(vec![
                PromptChoice::new().effect(GameEffect::EndRaid(s.ability_id())),
                PromptChoice::new().effect(GameEffect::TakeDamageCost(s.ability_id(), 2)),
            ])
        }))],
        config: CardConfigBuilder::new()
            .health(3)
            .shield(2)
            .resonance(Resonance::infernal())
            .build(),
    }
}
