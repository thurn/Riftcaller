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
use card_helpers::{
    combat_abilities, costs, delegates, requirements, show_prompt, text, text_helpers, this,
};
use core_data::game_primitives::{
    CardSubtype, CardType, GameObjectId, ManaValue, Rarity, School, Side,
};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_configuration::{CardConfigBuilder, Resonance};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::{CardIdsExt, CardPosition};
use game_data::delegate_data::RaidOutcome;
use game_data::game_effect::GameEffect;
use game_data::prompt_data::{PromptChoice, PromptChoiceLabel, PromptData};
use game_data::special_effects::{
    Projectile, ProjectileData, SoundEffect, TimedEffect, TimedEffectData,
};
use game_data::text::TextToken::*;
use game_data::utils;
use rules::visual_effects::{ShowAlert, VisualEffects};
use rules::{end_raid, mana, prompts, visual_effects};
use with_error::fail;

pub fn incarnation_of_justice(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::IncarnationOfJustice,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::covenant_card(meta, "incarnation_of_justice"),
        card_type: CardType::Minion,
        subtypes: vec![CardSubtype::Fey],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Rare,
        abilities: vec![Ability::new_with_delegate(
            text_helpers::named_trigger(
                Combat,
                text!["The Riftcaller cannot draw cards this turn"],
            ),
            delegates::on_will_draw_cards(
                requirements::combat_ability_fired_this_turn,
                |g, s, _| {
                    let Some(state) = g.state_machines.draw_cards.last_mut() else {
                        fail!("Expected active draw_cards state machine");
                    };
                    state.draw_is_prevented = true;

                    VisualEffects::new()
                        .ability_alert(s)
                        .timed_effect(
                            GameObjectId::CardId(s.card_id()),
                            TimedEffectData::new(TimedEffect::MagicCircles1(7))
                                .scale(2.0)
                                .sound(SoundEffect::LightMagic(
                                    "RPG3_LightMagicMisc_AttackMissed04",
                                ))
                                .effect_color(design::YELLOW_900),
                        )
                        .apply(g);
                    Ok(())
                },
            ),
        )],
        config: CardConfigBuilder::new()
            .health(5)
            .shield(meta.upgrade(1, 3))
            .resonance(Resonance::mortal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(4))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic3_Projectile01"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_Impact01")),
            )
            .build(),
    }
}

pub fn sentinel_sphinx(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SentinelSphinx,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(4),
        image: assets::covenant_card(meta, "sentinel_sphinx"),
        card_type: CardType::Minion,
        subtypes: vec![CardSubtype::Beast],
        side: Side::Covenant,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text!["This minion cannot be", Evaded],
                this::can_evade(delegates::disallow),
            ),
            combat_abilities::end_raid(),
        ],
        config: CardConfigBuilder::new()
            .health(meta.upgrade(2, 3))
            .shield(meta.upgrade(1, 2))
            .resonance(Resonance::infernal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(6))
                    .fire_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Projectiles01"))
                    .impact_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Impact01")),
            )
            .build(),
    }
}

pub fn lawhold_cavalier(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::LawholdCavalier,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(2),
        image: assets::covenant_card(meta, "lawhold_cavalier"),
        card_type: CardType::Minion,
        subtypes: vec![CardSubtype::Humanoid],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Rare,
        abilities: vec![
            Ability::new(text_helpers::named_trigger(
                Encounter,
                text!["The Riftcaller cannot play permanents this turn"],
            ))
            .delegate(delegates::can_play_card(
                requirements::combat_ability_fired_this_turn,
                |g, _, &card_id, flag| {
                    flag.prevent_if(
                        card_id.side == Side::Riftcaller
                            && g.card(card_id).definition().is_permanent(),
                    )
                },
            )),
            Ability::new(text_helpers::named_trigger(
                Combat,
                text![
                    text!["Choose", 2, "Riftcaller permanents, if able"],
                    text!["The Riftcaller must return one of them to the top of their deck"]
                ],
            ))
            .delegate(this::combat(|g, s, _| {
                let permanents = g.all_permanents(Side::Riftcaller).card_ids();
                if permanents.len() > 1 {
                    visual_effects::show(
                        g,
                        s,
                        GameObjectId::Character(Side::Covenant),
                        ShowAlert::Yes,
                    );
                }

                if permanents.len() == 2 {
                    g.covenant.prompt_selected_cards.push(permanents[0]);
                    g.covenant.prompt_selected_cards.push(permanents[1]);
                    prompts::push_with_data(g, Side::Riftcaller, s, PromptData::Index(2));
                } else if permanents.len() > 2 {
                    // Note that second option is shown first on prompt stack
                    prompts::push_with_data(g, Side::Covenant, s, PromptData::Index(1));
                    prompts::push_with_data(g, Side::Covenant, s, PromptData::Index(0));
                }

                Ok(())
            }))
            .delegate(this::prompt(|g, s, source, _| {
                let permanents = g.all_permanents(Side::Riftcaller).card_ids();
                let PromptData::Index(index) = source.data else {
                    return None;
                };

                match index {
                    0 => show_prompt::with_choices(
                        permanents
                            .into_iter()
                            .map(|card_id| {
                                PromptChoice::new()
                                    .effect(GameEffect::SelectCardForPrompt(
                                        Side::Covenant,
                                        card_id,
                                    ))
                                    .anchor_card(card_id)
                            })
                            .collect(),
                    ),
                    1 => show_prompt::with_choices(
                        permanents
                            .into_iter()
                            .filter(|card_id| !g.covenant.prompt_selected_cards.contains(&card_id))
                            .map(|card_id| {
                                PromptChoice::new()
                                    .effect(GameEffect::SelectCardForPrompt(
                                        Side::Covenant,
                                        card_id,
                                    ))
                                    .effect(GameEffect::PushPromptWithIndex(
                                        Side::Riftcaller,
                                        s.ability_id(),
                                        2,
                                    ))
                                    .anchor_card(card_id)
                                    .custom_label(PromptChoiceLabel::Select)
                            })
                            .collect(),
                    ),
                    2 => show_prompt::with_choices(
                        g.covenant
                            .prompt_selected_cards
                            .iter()
                            .map(|&card_id| {
                                PromptChoice::new()
                                    .effect(GameEffect::MoveCard(
                                        card_id,
                                        CardPosition::DeckTop(Side::Riftcaller),
                                    ))
                                    .effect(GameEffect::ClearAllSelectedCards(Side::Covenant))
                                    .anchor_card(card_id)
                                    .custom_label(PromptChoiceLabel::Return)
                            })
                            .collect(),
                    ),
                    _ => None,
                }
            })),
        ],
        config: CardConfigBuilder::new()
            .health(meta.upgrade(3, 5))
            .shield(1)
            .resonance(Resonance::infernal())
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles1(7))
                    .scale(1.5)
                    .effect_color(design::YELLOW_900)
                    .sound(SoundEffect::LightMagic("RPG3_LightMagic_Cast02")),
            )
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(4))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic2_Projectile01"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagic_Impact01")),
            )
            .build(),
    }
}

pub fn angel_of_unity(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::AngelOfUnity,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(4),
        image: assets::covenant_card(meta, "angel_of_unity"),
        card_type: CardType::Minion,
        subtypes: vec![CardSubtype::Roombound, CardSubtype::Fey],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Rare,
        abilities: vec![
            ActivatedAbility::new(
                costs::sacrifice(),
                text![
                    text!["End the raid"],
                    text!["Use this ability only during a raid on this room"]
                ],
            )
            .delegate(this::can_activate(|g, s, _, flag| {
                flag.add_constraint(utils::is_true(|| {
                    Some(g.raid.as_ref()?.target == g.card(s).position().defending_room()?)
                }))
            }))
            .delegate(this::on_activated(|g, s, _| {
                end_raid::run(g, s.initiated_by(), RaidOutcome::Failure)
            }))
            .build(),
            Ability::new(text_helpers::named_trigger(
                Combat,
                text![
                    text![GainMana(meta.upgrade(1, 2)), "for each minion defending this room"],
                    text!["End the raid"]
                ],
            ))
            .delegate(this::combat(|g, s, _| {
                let Some(room_id) = g.card(s).position().defending_room() else {
                    return Ok(());
                };
                visual_effects::show_alert(g, s);
                mana::gain(
                    g,
                    Side::Covenant,
                    g.defenders_unordered(room_id).count() as ManaValue * s.upgrade(1, 2),
                );
                end_raid::run(g, s.initiated_by(), RaidOutcome::Failure)
            })),
        ],
        config: CardConfigBuilder::new()
            .health(1)
            .shield(2)
            .resonance(Resonance::astral())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles2(7))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic2_Projectile01"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagic_Impact01")),
            )
            .build(),
    }
}
