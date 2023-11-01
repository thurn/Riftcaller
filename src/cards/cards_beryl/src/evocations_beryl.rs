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

use card_helpers::abilities::ActivatedConfig;
use card_helpers::effects::Effects;
use card_helpers::{
    abilities, costs, delegates, in_play, raids, requirements, show_prompt, text, this,
};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_definition::{
    Ability, AbilityType, CardConfig, CardDefinition, TargetRequirement,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::game_actions::{ButtonPromptContext, PromptChoice};
use game_data::game_effect::GameEffect;
use game_data::game_state::RaidJumpRequest;
use game_data::primitives::{CardSubtype, CardType, GameObjectId, Rarity, School, Side};
use game_data::raid_data::RaidStatus;
use game_data::special_effects::{SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextElement;
use game_data::text::TextToken::*;
use game_data::utils;
use rules::{flags, mana, mutations, queries, CardDefinitionExt};

pub fn empyreal_chorus(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EmpyrealChorus,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(1),
        image: assets::champion_card(meta, "empyreal_chorus"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            ability_type: AbilityType::Activated {
                cost: costs::sacrifice_for_action(),
                target_requirement: TargetRequirement::TargetRoom(|g, _, r| {
                    r.is_outer_room() && flags::is_valid_raid_target(g, r)
                }),
                can_activate: None,
            },
            text: text![
                text!["Raid target outer room"],
                text![
                    "If successful",
                    Gain,
                    Mana(meta.upgrade(8, 10)),
                    "instead of accessing cards"
                ]
            ],
            delegates: vec![
                this::on_activated(|g, s, activated| raids::initiate(g, s, activated.target)),
                delegates::can_raid_access_cards(requirements::matching_raid, |_, _, _, flag| {
                    flag.with_override(false)
                }),
                delegates::on_raid_successful(requirements::matching_raid, |g, s, _| {
                    Effects::new()
                        .ability_alert(s)
                        .timed_effect(
                            GameObjectId::Character(Side::Champion),
                            TimedEffectData::new(TimedEffect::MagicCircles1(10))
                                .scale(4.0)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagic_Cast02"))
                                .effect_color(design::YELLOW_900),
                        )
                        .apply(g);
                    mana::gain(g, s.side(), s.upgrade(8, 10));
                    Ok(())
                }),
            ],
        }],
        config: CardConfig::default(),
    }
}

pub fn starfield_omen(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::StarfieldOmen,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(4),
        image: assets::champion_card(meta, "starfield_omen"),
        card_type: CardType::Evocation,
        subtypes: vec![CardSubtype::Mystic],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::standard(
            text!["When you sacrifice an artifact, draw a card"],
            in_play::on_card_sacrificed(|g, s, card_id| {
                if g.card(*card_id).definition().card_type == CardType::Artifact {
                    Effects::new()
                        .timed_effect(
                            GameObjectId::CardId(s.card_id()),
                            TimedEffectData::new(TimedEffect::MagicCircles2(12))
                                .scale(1.5)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_Transform01"))
                                .effect_color(design::YELLOW_900),
                        )
                        .apply(g);

                    mutations::draw_cards(g, s.side(), 1)?;
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
        image: assets::champion_card(meta, "visitation"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::standard(
            // This is templated as an activated ability for clarity even though it's
            // secretly not.
            text![TextElement::Activated {
                cost: text![SacrificeCost],
                effect: text!["Prevent up to", meta.upgrade(2, 5), Damage]
            }],
            in_play::on_will_deal_damage(|g, s, damage| {
                if damage.source.side() == Side::Overlord {
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
        image: assets::champion_card(meta, "backup_plan"),
        card_type: CardType::Evocation,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![abilities::activated_with_config(
            text![
                text![Evade, "a minion"],
                meta.upgrade(text!["Lose all", ActionSymbol], text!["Lose", Actions(1)])
            ],
            costs::sacrifice(),
            ActivatedConfig::new().can_activate(|g, _| {
                utils::is_true(|| {
                    Some(queries::raid_status(g.raid.as_ref()?) == RaidStatus::Encounter)
                })
            }),
            this::on_activated(|g, s, _| {
                g.raid_mut()?.jump_request = Some(RaidJumpRequest::EvadeCurrentMinion);
                mutations::lose_action_points_if_able(
                    g,
                    Side::Champion,
                    s.upgrade(g.champion.actions, 1),
                )?;
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}
